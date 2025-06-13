use super::Scene;
use super::selectsave::{SelectSave, preview_width_for_screen_width};
use crate::{
    cursor::Cursor,
    internal::files_for_directory,
    manager::Action,
    ui::{
        Button,
        screen::{Color, FontSize, Rect, SHADOW_DELTA, Screen},
    },
};
use rand::Rng;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const PADDING: u32 = 4;
pub const PAGE_SIZE: usize = 10;

pub struct SelectGame {
    root: PathBuf,
    destination: PathBuf,
    games: Vec<PathBuf>,
    cursor: Cursor,
    offset: usize,
}

pub enum Operation {
    ExecGame(PathBuf),
}

fn preview_image_for_game(game: &Path) -> Option<PathBuf> {
    files_for_directory(game)
        .filter(|p| matches!(p.extension().and_then(|p| p.to_str()), Some("png" | "jpg")))
        .last()
}

impl SelectGame {
    pub fn new(root: PathBuf, destination: PathBuf) -> Self {
        let games = WalkDir::new(&root)
            .min_depth(3)
            .max_depth(3)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
            .map(|e| e.into_path())
            .collect::<Vec<_>>();

        let offset = rand::rng().random_range(100..999);
        let len = games.len();

        Self {
            root,
            destination,
            games,
            offset,
            cursor: Cursor::new(len, PAGE_SIZE),
        }
    }

    fn label_for(&self, game: &Path) -> String {
        game.strip_prefix(&self.root)
            .unwrap()
            .iter()
            .skip(1)
            .enumerate()
            .map(|(i, p)| {
                if i == 0 {
                    match p.to_str().unwrap() {
                        "Japanese" => "(J)",
                        "Chinese" => "(C)",
                        "English" => "(E)",
                        _ => "(O)",
                    }
                } else {
                    p.to_str().unwrap()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn current_game(&self) -> Option<&Path> {
        self.games.get(self.cursor.index()).map(|p| p.as_path())
    }
}

impl Scene<Operation> for SelectGame {
    fn pressed(&mut self, button: &Button) -> Action<Operation> {
        match button {
            Button::B => return Action::Pop,
            Button::Up => {
                self.cursor.up();
            }
            Button::Down => {
                self.cursor.down();
            }
            Button::Left => {
                self.cursor.page_up();
            }
            Button::Right => {
                self.cursor.page_down();
            }
            Button::A => {
                let game = self.current_game().unwrap();
                let scene =
                    SelectSave::new(game.to_owned(), self.root.clone(), self.destination.clone());
                return Action::Push(Box::new(scene));
            }
            Button::Start => {
                if let Some(game) = self.current_game() {
                    return Action::Complete(Operation::ExecGame(game.to_owned()));
                }
            }
            _ => {}
        }

        Action::Continue
    }

    fn draw(&self, screen: &mut Screen) {
        let mut y = 0;

        let (_, font18height) = screen.measure_text(FontSize::Size18, "S");
        let (_, font14height) = screen.measure_text(FontSize::Size14, "0");

        let gap = screen.recommended_margin();
        let (screen_width, screen_height) = screen.size();
        let preview_width = preview_width_for_screen_width(screen_width);

        if let Some(path) = self.current_game().and_then(preview_image_for_game) {
            let background = Screen::create_background(&path, 128, 90);
            let i = (self.cursor.index() + self.offset) as f64;
            let scale = (2.0 + i.sin() / 2.0) as f32;
            let angle = i.cos() * 30.0;
            screen.draw_background(background, scale, angle);
        }

        let full_list_height =
            2 * gap + font18height + PAGE_SIZE as u32 * (font14height + PADDING * 2 - 1);
        let fill_height = screen_height - 2 * gap;
        let extra_gap = fill_height - full_list_height;

        let bg_height = 2 * gap
            + font18height
            + extra_gap
            + self.cursor.visible_items() as u32 * (font14height + PADDING * 2 - 1);

        screen.draw_rect(
            Color::RGBA(0, 0, 0, 64),
            Rect::new(
                gap as i32,
                gap as i32,
                screen_width - 5 * gap - preview_width,
                bg_height,
            ),
        );

        screen.draw_text(
            FontSize::Size18,
            "Select a game",
            2 * gap as i32,
            2 * gap as i32,
        );
        y += (2 * gap + font18height + extra_gap) as i32;

        for (selected, game) in self.cursor.iter(self.games.iter()) {
            if selected {
                screen.draw_rect(
                    Color::RGBA(0, 0, 255, 180),
                    Rect::new(
                        2 * gap as i32,
                        y - PADDING as i32,
                        screen_width - 7 * gap - preview_width,
                        14 + 2 * PADDING + SHADOW_DELTA,
                    ),
                );
            }

            let label = self.label_for(game);
            screen.draw_text_clipped(
                FontSize::Size14,
                &label,
                2 * gap as i32 + PADDING as i32,
                y,
                screen_width - 7 * gap - preview_width - 2 * PADDING,
            );
            y += (font14height + PADDING * 2 - 1) as i32;
        }
    }
}
