use super::Scene;
use crate::{
    manager::Action,
    ui::{
        Button,
        screen::{FontSize, Screen},
    },
};
use sdl2::{pixels::Color, rect::Rect};
use tracing::{error, info};

pub struct Message {
    messages: Vec<String>,
    is_error: bool,
}

impl Message {
    pub fn new(messages: Vec<String>, is_error: bool) -> Self {
        if is_error {
            error!("{messages:?}");
        } else {
            info!("{messages:?}");
        }

        Self { messages, is_error }
    }
}

impl Scene for Message {
    fn is_overlay(&self) -> bool {
        true
    }

    fn pressed(&mut self, button: &Button) -> Action {
        match button {
            Button::B => Action::Pop,
            Button::A => Action::Pop,
            Button::Start => Action::Bubble,
            _ => Action::Continue,
        }
    }

    fn draw(&self, screen: &mut Screen) {
        let (screen_width, screen_height) = screen.size();
        let gap = screen.recommended_margin();
        let mut text_width = 0;
        let mut text_height = 0;
        let mut messages = Vec::with_capacity(self.messages.len());

        screen.draw_rect(
            Color::RGBA(0, 0, 0, 128),
            Rect::new(0, 0, screen_width, screen_height),
        );

        for (i, text) in self.messages.iter().enumerate() {
            let (cap, size) = if i == 0 {
                (40, FontSize::Size18)
            } else {
                (70, FontSize::Size14)
            };
            let text = text.trim();
            let text = if text.len() > cap {
                format!("{}…", &text[0..cap])
            } else {
                text.to_string()
            };
            let (w, h) = screen.measure_text(size, &text);
            text_height += h + if i == 0 { 0 } else { gap };
            if w > text_width {
                text_width = w;
            }
            messages.push((text, w, h, size));
        }

        let box_x = (screen_width as i32 - text_width as i32) / 2;
        let box_y = (screen_height as i32 - text_height as i32) / 2;
        screen.draw_rect(
            Color::RGBA(0, 0, 0, 128),
            Rect::new(box_x, box_y, text_width + gap * 2, text_height + gap * 2),
        );

        screen.draw_rect(
            if self.is_error {
                Color::RGBA(96, 0, 0, 255)
            } else {
                Color::RGBA(36, 36, 36, 255)
            },
            Rect::new(
                box_x - gap as i32,
                box_y - gap as i32,
                text_width + gap * 2,
                text_height + gap * 2,
            ),
        );

        let mut y = box_y;
        for (text, w, h, size) in messages {
            let x = (screen_width as i32 - w as i32) / 2;
            screen.draw_text(size, &text, x, y);
            y += (h + gap) as i32;
        }
    }
}
