mod message;
mod selectgame;
mod selectsave;

use crate::internal::remove_full_extension;
use crate::ui;
use crate::ui::{screen::Screen, Button};
use log::error;
use sdl2::pixels::Color;
use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

enum Action<'a> {
    Continue,
    Push(Box<dyn Scene>),
    Pop,
    ExecGame(&'a Path),
}

pub struct Manager {
    scenes: Vec<Box<dyn Scene>>,
    exec_command: Option<PathBuf>,
}

impl Manager {
    pub fn new(root: PathBuf, destination: PathBuf, exec_command: Option<PathBuf>) -> Self {
        let games = walkdir::WalkDir::new(&root)
            .min_depth(3)
            .max_depth(3)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
            .map(|e| e.into_path())
            .collect();

        Self {
            scenes: vec![Box::new(selectgame::SelectGame::new(
                root,
                destination,
                games,
            ))],
            exec_command,
        }
    }

    pub fn pressed(&mut self, button: Button) -> ui::Action {
        let action = match button {
            Button::Select => return ui::Action::Quit,
            Button::R2 => return ui::Action::Screenshot,
            _ => self.scenes.last_mut().unwrap().pressed(button),
        };

        match action {
            Action::Continue => ui::Action::Continue,
            Action::Push(scene) => {
                self.scenes.push(scene);
                ui::Action::Continue
            }
            Action::Pop => {
                self.scenes.pop();
                if self.scenes.is_empty() {
                    ui::Action::Quit
                } else {
                    ui::Action::Continue
                }
            }
            Action::ExecGame(path) => {
                if let Some(command) = &self.exec_command {
                    let err = Command::new(command).arg(path).exec();
                    error!("Error exec'ing {:?}: {err}", command);
                    ui::Action::Quit
                } else {
                    ui::Action::Continue
                }
            }
        }
    }

    pub fn background_color(&self) -> Color {
        for scene in self.scenes.iter().rev() {
            if !scene.is_overlay() {
                return scene.background_color();
            }
        }
        unreachable!()
    }

    pub fn draw(&self, screen: &mut Screen) {
        let mut scenes = Vec::new();
        for scene in self.scenes.iter().rev() {
            scenes.push(scene);
            if !scene.is_overlay() {
                break;
            }
        }

        for scene in scenes.iter().rev() {
            scene.draw(screen);
        }
    }
}

trait Scene {
    fn pressed(&mut self, button: Button) -> Action;

    fn draw(&self, screen: &mut Screen);

    fn is_overlay(&self) -> bool {
        false
    }

    fn background_color(&self) -> Color {
        Color::RGB(0, 0, 0)
    }
}

fn files_for_game(game: &Path) -> impl Iterator<Item = PathBuf> {
    walkdir::WalkDir::new(game)
        .min_depth(1)
        .max_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
}

fn preview_image_for_game(game: &Path) -> Option<PathBuf> {
    files_for_game(game)
        .filter(|p| matches!(p.extension().and_then(|p| p.to_str()), Some("png" | "jpg")))
        .last()
}

fn saves_for_game(game: &Path) -> Vec<(PathBuf, Option<PathBuf>)> {
    let mut images: HashMap<std::ffi::OsString, PathBuf> = HashMap::new();
    let mut saves = Vec::new();

    for file in files_for_game(game) {
        match file.extension().and_then(|p| p.to_str()) {
            Some("png" | "jpg") => {
                let mut stem = file.clone();
                remove_full_extension(&mut stem);
                images.insert(stem.into_os_string(), file);
            }
            Some(_) => {
                saves.push(file);
            }
            None => {}
        }
    }

    saves
        .into_iter()
        .map(move |p| {
            let mut image = p.clone();
            remove_full_extension(&mut image);
            let image = images.get(&image.into_os_string()).cloned();
            (p, image)
        })
        .collect()
}
