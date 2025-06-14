pub mod message;
pub mod selectgame;
pub mod selectsave;
pub mod selectstring;

use crate::{
    manager::Action,
    ui::{Button, Color, screen::Screen},
};
use std::path::Path;

pub trait Scene<T> {
    fn pressed(&mut self, button: &Button) -> Option<Action<T>>;

    fn draw(&self, screen: &mut Screen);

    fn is_overlay(&self) -> bool {
        false
    }

    fn background_color(&self) -> Color {
        Color::RGB(0, 0, 0)
    }

    fn draw_stylized_background(
        &self,
        screen: &mut Screen,
        image_path: &Path,
        n: usize,
    ) -> (u32, u32) {
        let background = Screen::create_background(image_path, 128, 90);
        let (width, height) = background.size();
        let i = n as f64;
        let scale = (2.0 + i.sin() / 2.0) as f32;
        let angle = i.cos() * 30.0;
        screen.draw_background(background, scale, angle);
        (width, height)
    }
}
