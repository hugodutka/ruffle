//! MovieClip prototype

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::display_object::{MovieClip, TDisplayObject};
use enumset::EnumSet;
use gc_arena::MutationContext;

/// Implements `MovieClip`
pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

macro_rules! with_movie_clip {
    ( $gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |avm, context: &mut UpdateContext<'_, 'gc, '_>, this, args| -> Result<ReturnValue<'gc>, Error> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(movie_clip) = display_object.as_movie_clip() {
                            return $fn(movie_clip, avm, context, args);
                        }
                    }
                    Ok(Value::Undefined.into())
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
                $fn_proto
            );
        )*
    }};
}

pub fn overwrite_root<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(ac.gc_context, "_root", new_val, EnumSet::new());

    Ok(Value::Undefined.into())
}

pub fn overwrite_global<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(ac.gc_context, "_global", new_val, EnumSet::new());

    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    with_movie_clip!(
        gc_context,
        object,
        Some(fn_proto),
        "nextFrame" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            movie_clip.next_frame(context);
            Ok(Value::Undefined.into())
        },
        "prevFrame" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            movie_clip.prev_frame(context);
            Ok(Value::Undefined.into())
        },
        "play" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            movie_clip.play(context);
            Ok(Value::Undefined.into())
        },
        "stop" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            movie_clip.stop(context);
            Ok(Value::Undefined.into())
        },
        "getBytesLoaded" => |_movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            // TODO find a correct value
            Ok(1.0.into())
        },
        "getBytesTotal" => |_movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            // TODO find a correct value
            Ok(1.0.into())
        },
        "gotoAndPlay" => goto_and_play,
        "gotoAndStop" => goto_and_stop,
        "toString" => |movie_clip: MovieClip<'gc>, _avm: &mut Avm1<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            Ok(movie_clip.path().into())
        }
    );

    object.add_property(
        gc_context,
        "_global",
        Executable::Native(|avm, context, _this, _args| Ok(avm.global_object(context).into())),
        Some(Executable::Native(overwrite_global)),
        DontDelete | ReadOnly | DontEnum,
    );

    object.add_property(
        gc_context,
        "_root",
        Executable::Native(|avm, context, _this, _args| Ok(avm.root_object(context).into())),
        Some(Executable::Native(overwrite_root)),
        DontDelete | ReadOnly | DontEnum,
    );

    object.add_property(
        gc_context,
        "_parent",
        Executable::Native(|_avm, _context, this, _args| {
            Ok(this
                .as_display_object()
                .and_then(|mc| mc.parent())
                .and_then(|dn| dn.object().as_object().ok())
                .map(Value::Object)
                .unwrap_or(Value::Undefined)
                .into())
        }),
        None,
        DontDelete | ReadOnly | DontEnum,
    );

    object.into()
}

pub fn goto_and_play<'gc>(
    movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    goto_frame(movie_clip, avm, context, args, false)
}

pub fn goto_and_stop<'gc>(
    movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    goto_frame(movie_clip, avm, context, args, true)
}

#[allow(clippy::unreadable_literal)]
pub fn goto_frame<'gc>(
    movie_clip: MovieClip<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
    stop: bool,
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(value) = args.get(0) {
        if let Ok(mut frame) = value.as_i32() {
            // Frame #
            // Gotoing <= 0 has no effect.
            // Gotoing greater than _totalframes jumps to the last frame.
            // Wraps around as an i32.
            // TODO: -1 +1 here to match Flash's behavior.
            // We probably want to change our frame representation to 0-based.
            frame = frame.wrapping_sub(1);
            if frame >= 0 {
                let num_frames = movie_clip.total_frames();
                if frame > i32::from(num_frames) {
                    movie_clip.goto_frame(context, num_frames, stop);
                } else {
                    movie_clip.goto_frame(context, frame.saturating_add(1) as u16, stop);
                }
            }
        } else {
            let frame_label = value.clone().coerce_to_string(avm, context)?;
            if let Some(frame) = movie_clip.frame_label_to_number(&frame_label) {
                movie_clip.goto_frame(context, frame, stop);
            }
        }
    }
    Ok(Value::Undefined.into())
}