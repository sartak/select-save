use crate::{
    scene::Scene,
    ui::{
        self, Button,
        screen::{Color, Screen},
    },
};

pub enum Action<T> {
    Continue,
    Push(Box<dyn Scene<T>>),
    Pop,
    Complete(T),
    Bubble,
}

pub struct Manager<T> {
    scenes: Vec<Box<dyn Scene<T>>>,
}

impl<T> Manager<T> {
    pub fn new(root_scene: Box<dyn Scene<T>>) -> Self {
        Self {
            scenes: vec![root_scene],
        }
    }

    pub fn pressed(&mut self, button: Button) -> ui::Action<T> {
        let action = match button {
            Button::Select => return ui::Action::Cancel,
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
            return ui::Action::Cancel;
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
                    ui::Action::Cancel
                } else {
                    ui::Action::Continue
                }
            }
            Action::Complete(t) => ui::Action::Complete(t),
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
