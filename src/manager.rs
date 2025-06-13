use crate::{
    scene::Scene,
    ui::{self, Button, screen::Screen},
};
use sdl2::pixels::Color;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::error;

pub enum Action<'a> {
    Continue,
    Push(Box<dyn Scene>),
    Pop,
    ExecGame(&'a Path),
    Bubble,
}

pub struct Manager {
    scenes: Vec<Box<dyn Scene>>,
    exec_command: Option<PathBuf>,
}

impl Manager {
    pub fn new(root_scene: Box<dyn Scene>, exec_command: Option<PathBuf>) -> Self {
        Self {
            scenes: vec![root_scene],
            exec_command,
        }
    }

    pub fn pressed(&mut self, button: Button) -> ui::Action {
        let action = match button {
            Button::Select => return ui::Action::Quit,
            Button::R2 => return ui::Action::Screenshot,
            _ => self
                .scenes
                .iter_mut()
                .rev()
                .find_map(|scene| match scene.pressed(&button) {
                    Action::Bubble => None,
                    a => Some(a),
                }),
        };

        let Some(action) = action else {
            return ui::Action::Quit;
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
            Action::Bubble => unreachable!(),
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
