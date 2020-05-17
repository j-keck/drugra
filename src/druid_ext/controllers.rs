use druid::widget::Controller;
use druid::{Env, Event, EventCtx, KeyCode, Widget};

/// Focus the child widget on launch
pub(crate) struct FocusOnLaunchCtrl;
impl FocusOnLaunchCtrl {
    pub fn new() -> Self {
        Self {}
    }
}
impl<T, W: Widget<T>> Controller<T, W> for FocusOnLaunchCtrl {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::WindowConnected = event {
            ctx.request_focus();
        }
        child.event(ctx, event, data, env);
    }
}

/// Execute the given action when the `Return` key is pressed.
pub(crate) struct ActionCtrl<T> {
    f: Box<dyn FnMut(&mut T, &mut EventCtx) -> ()>,
}
impl<T> ActionCtrl<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T, &mut EventCtx) -> () + Sized + 'static,
    {
        Self { f: Box::new(f) }
    }
}
impl<T, W: Widget<T>> Controller<T, W> for ActionCtrl<T> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::KeyDown(k) = event {
            if k.key_code == KeyCode::Return {
                (self.f)(data, ctx);
            }
        }
        child.event(ctx, event, data, env);
    }
}

/// Input event filter over key codes.
pub(crate) struct KeyCodeFilterCtrl {
    is_valid: Box<dyn FnMut(&KeyCode) -> bool>,
}
impl KeyCodeFilterCtrl {
    /// Create a new `KeyCodeFilterCtrl` with the provided predicate function.
    pub fn new<F>(is_valid: F) -> Self
    where
        F: FnMut(&KeyCode) -> bool + 'static,
    {
        Self {
            is_valid: Box::new(is_valid),
        }
    }

    ///
    pub fn with_meta_keys(mut self) -> Self {
        use KeyCode::*;
        let meta_keys = vec![ArrowLeft, ArrowRight, Backspace, Delete];
        Self::new(move |code| (self.is_valid)(&code) || meta_keys.contains(&code))
    }
}
impl<T, W: Widget<T>> Controller<T, W> for KeyCodeFilterCtrl {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::KeyDown(k) = event {
            if (self.is_valid)(&k.key_code) {
                child.event(ctx, event, data, env)
            }
        } else {
            child.event(ctx, event, data, env)
        }
    }
}
