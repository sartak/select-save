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

pub struct List<T> {
    items: Vec<T>,
    cursor: Cursor,
    title: String,
}

struct Layout {
    gap: u32,
    title_font_height: u32,
    body_font_height: u32,
    screen_height: u32,
    background_width_with_preview: u32,
    background_width_no_preview: u32,
    list_x: i32,
}

pub fn preview_width_for_screen_width(width: u32) -> u32 {
    width / 3
}

impl<T> List<T> {
    pub fn new(items: Vec<T>, title: String) -> Self {
        let len = items.len();
        Self {
            items,
            cursor: Cursor::new(len, PAGE_SIZE),
            title,
        }
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn current_item(&self) -> Option<&T> {
        self.items.get(self.cursor.index())
    }

    pub fn handle_navigation<U>(&mut self, button: &Button) -> Option<Action<U>> {
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
            _ => None,
        }
    }

    fn calculate_layout(&self, screen: &mut Screen) -> Layout {
        let (_, title_font_height) = screen.measure_text(FontSize::Title, "S");
        let (_, body_font_height) = screen.measure_text(FontSize::Body, "0");

        let gap = screen.recommended_margin();
        let (screen_width, screen_height) = screen.size();
        let preview_width = preview_width_for_screen_width(screen_width);

        let background_width_no_preview = screen_width - 5 * gap;
        let background_width_with_preview = background_width_no_preview - preview_width;

        Layout {
            gap,
            title_font_height,
            body_font_height,
            screen_height,
            background_width_with_preview,
            background_width_no_preview,
            list_x: 2 * gap as i32,
        }
    }

    fn draw_background(&self, screen: &mut Screen, layout: &Layout, preview: bool) {
        let background_width = if preview {
            layout.background_width_with_preview
        } else {
            layout.background_width_no_preview
        };

        let full_list_height = 2 * layout.gap
            + layout.title_font_height
            + PAGE_SIZE as u32 * (layout.body_font_height + PADDING * 2 - 1);
        let fill_height = layout.screen_height - 2 * layout.gap;
        let extra_gap = fill_height - full_list_height;

        let bg_height = 2 * layout.gap
            + layout.title_font_height
            + extra_gap
            + self.cursor.visible_items() as u32 * (layout.body_font_height + PADDING * 2 - 1);

        screen.draw_rect(
            Color::RGBA(0, 0, 0, 64),
            Rect::new(
                layout.gap as i32,
                layout.gap as i32,
                background_width,
                bg_height,
            ),
        );
    }

    fn draw_title(&self, screen: &mut Screen, layout: &Layout) -> i32 {
        screen.draw_text(
            FontSize::Title,
            &self.title,
            layout.list_x,
            2 * layout.gap as i32,
        );

        let full_list_height = 2 * layout.gap
            + layout.title_font_height
            + PAGE_SIZE as u32 * (layout.body_font_height + PADDING * 2 - 1);
        let fill_height = layout.screen_height - 2 * layout.gap;
        let extra_gap = fill_height - full_list_height;

        (2 * layout.gap + layout.title_font_height + extra_gap) as i32
    }

    fn draw_list<F>(
        &self,
        screen: &mut Screen,
        layout: &Layout,
        preview: bool,
        mut y: i32,
        label_fn: F,
    ) where
        F: Fn(&T) -> String,
    {
        let background_width = if preview {
            layout.background_width_with_preview
        } else {
            layout.background_width_no_preview
        };

        for (selected, item) in self.cursor.iter(self.items.iter()) {
            if selected {
                screen.draw_rect(
                    Color::RGBA(0, 0, 255, 180),
                    Rect::new(
                        layout.list_x,
                        y - PADDING as i32,
                        background_width - 2 * layout.gap,
                        14 + 2 * PADDING + SHADOW_DELTA,
                    ),
                );
            }

            let label = label_fn(item);
            screen.draw_text_clipped(
                FontSize::Body,
                &label,
                layout.list_x + PADDING as i32,
                y,
                background_width - 2 * layout.gap - 2 * PADDING,
            );
            y += (layout.body_font_height + PADDING * 2 - 1) as i32;
        }
    }

    pub fn draw<F>(&self, screen: &mut Screen, preview: bool, label_fn: F)
    where
        F: Fn(&T) -> String,
    {
        let layout = self.calculate_layout(screen);
        self.draw_background(screen, &layout, preview);
        let y = self.draw_title(screen, &layout);
        self.draw_list(screen, &layout, preview, y, label_fn);
    }
}
