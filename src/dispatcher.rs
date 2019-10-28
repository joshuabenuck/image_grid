use ggez::event::{self, KeyCode, KeyMods, MouseButton};
use ggez::{self, Context, GameResult};

pub trait Widget: event::EventHandler {
    fn next(&self) -> NextAction;
}

pub enum NextAction {
    None,
    Push(Box<dyn Widget>),
    Pop,
    Replace(Box<dyn Widget>),
}

pub struct Dispatcher {
    pub widget: Box<dyn Widget>,
    pub parent: Option<Box<dyn Widget>>,
}

impl event::EventHandler for Dispatcher {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let value = (*self.widget).update(ctx);
        match (*self.widget).next() {
            NextAction::None => (),
            NextAction::Push(widget) => {
                self.parent = Some(std::mem::replace(&mut self.widget, widget));
            }
            NextAction::Pop => {
                //std::mem::replace(&mut self.parent, ...)
            }
            NextAction::Replace(widget) => self.widget = widget,
        }
        value
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        (*self.widget).draw(ctx)
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, _button: MouseButton, x: f32, y: f32) {
        (*self.widget).mouse_button_up_event(ctx, _button, x, y)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        (*self.widget).mouse_wheel_event(_ctx, _x, y)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        (*self.widget).key_down_event(_ctx, keycode, _keymod, _repeat)
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        (*self.widget).key_up_event(_ctx, keycode, _keymod)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        (*self.widget).resize_event(ctx, width, height)
    }
}
