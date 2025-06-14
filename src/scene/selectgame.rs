use super::Scene;
use super::selectsave::SelectSave;
use crate::{
    internal::files_for_directory,
    manager::Action,
    ui::{Button, list::List, screen::Screen},
};
use rand::Rng;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct SelectGame {
    root: PathBuf,
    destination: PathBuf,
    list: List<PathBuf>,
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
        let list = List::new(games, "Select a game".to_string());

        Self {
            root,
            destination,
            list,
            offset,
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
        self.list.current_item().map(|p| p.as_path())
    }
}

impl Scene<Operation> for SelectGame {
    fn pressed(&mut self, button: &Button) -> Option<Action<Operation>> {
        if let Some(action) = self.list.handle_navigation(button) {
            return Some(action);
        }

        match button {
            Button::A => {
                let game = self.current_game().unwrap();
                let scene =
                    SelectSave::new(game.to_owned(), self.root.clone(), self.destination.clone());
                Some(Action::Push(Box::new(scene)))
            }
            Button::Start => {
                if let Some(game) = self.current_game() {
                    Some(Action::Complete(Operation::ExecGame(game.to_owned())))
                } else {
                    Some(Action::Continue)
                }
            }
            _ => Some(Action::Continue),
        }
    }

    fn draw(&self, screen: &mut Screen) {
        if let Some(path) = self.current_game().and_then(preview_image_for_game) {
            self.draw_stylized_background(screen, &path, self.list.cursor().index() + self.offset);
        }

        self.list
            .draw(screen, true, true, |game| self.label_for(game));
    }
}
