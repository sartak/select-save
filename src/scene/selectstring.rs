use super::Scene;
use crate::{
    cursor::Cursor,
    manager::Action,
    ui::{
        Button,
        screen::{Color, FontSize, Rect, SHADOW_DELTA, Screen},
    },
};

pub const PADDING: u32 = 4;
pub const PAGE_SIZE: usize = 10;

pub struct SelectString {
    items: Vec<String>,
    cursor: Cursor,
    title: String,
}

pub enum Operation {
    SelectItem(String),
}

impl SelectString {
    pub fn new(items: Vec<String>, title: String) -> Self {
        let len = items.len();
        Self {
            items,
            cursor: Cursor::new(len, PAGE_SIZE),
            title,
        }
    }
}

impl Scene<Operation> for SelectString {
    fn pressed(&mut self, button: &Button) -> Option<Action<Operation>> {
        match button {
            Button::B => Some(Action::Pop),
            Button::Up => {
                self.cursor.up();
                Some(Action::Continue)
            }
            Button::Down => {
                self.cursor.down();
                Some(Action::Continue)
            }
            Button::Left => {
                self.cursor.page_up();
                Some(Action::Continue)
            }
            Button::Right => {
                self.cursor.page_down();
                Some(Action::Continue)
            }
            Button::A => {
                if let Some(item) = self.items.get(self.cursor.index()) {
                    Some(Action::Complete(Operation::SelectItem(item.clone())))
                } else {
                    Some(Action::Continue)
                }
            }
            _ => Some(Action::Continue),
        }
    }

    fn draw(&self, screen: &mut Screen) {
        let mut y = 0;
        let (_, font18height) = screen.measure_text(FontSize::Size18, "S");
        let (_, font14height) = screen.measure_text(FontSize::Size14, "0");
        let gap = screen.recommended_margin();
        let (screen_width, screen_height) = screen.size();

        // Background
        let full_list_height =
            2 * gap + font18height + PAGE_SIZE as u32 * (font14height + PADDING * 2 - 1);
        let fill_height = screen_height - 2 * gap;
        let extra_gap = fill_height.saturating_sub(full_list_height);

        let bg_height = 2 * gap
            + font18height
            + extra_gap
            + self.cursor.visible_items() as u32 * (font14height + PADDING * 2 - 1);

        screen.draw_rect(
            Color::RGBA(0, 0, 0, 64),
            Rect::new(gap as i32, gap as i32, screen_width - 2 * gap, bg_height),
        );

        // Title
        screen.draw_text(
            FontSize::Size18,
            &self.title,
            2 * gap as i32,
            2 * gap as i32,
        );
        y += (2 * gap + font18height + extra_gap) as i32;

        // Items
        for (selected, item) in self.cursor.iter(self.items.iter()) {
            if selected {
                screen.draw_rect(
                    Color::RGBA(0, 0, 255, 180),
                    Rect::new(
                        2 * gap as i32,
                        y - PADDING as i32,
                        screen_width - 4 * gap,
                        14 + 2 * PADDING + SHADOW_DELTA,
                    ),
                );
            }

            screen.draw_text_clipped(
                FontSize::Size14,
                item,
                2 * gap as i32 + PADDING as i32,
                y,
                screen_width - 4 * gap - 2 * PADDING,
            );
            y += (font14height + PADDING * 2 - 1) as i32;
        }
    }
}
