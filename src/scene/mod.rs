pub mod message;
pub mod selectgame;
pub mod selectsave;

use crate::{
    manager::Action,
    ui::{Button, screen::Screen},
};
use sdl2::pixels::Color;

pub trait Scene {
    fn pressed(&mut self, button: &Button) -> Action;

    fn draw(&self, screen: &mut Screen);

    fn is_overlay(&self) -> bool {
        false
    }

    fn background_color(&self) -> Color {
        Color::RGB(0, 0, 0)
    }
}
