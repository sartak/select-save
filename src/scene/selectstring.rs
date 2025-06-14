use super::Scene;
use crate::{
    manager::Action,
    ui::{Button, list::List, screen::Screen},
};

pub struct SelectString {
    list: List<String>,
}

pub enum Operation {
    SelectItem(String),
}

impl SelectString {
    pub fn new(items: Vec<String>, title: String) -> Self {
        Self {
            list: List::new(items, title),
        }
    }
}

impl Scene<Operation> for SelectString {
    fn pressed(&mut self, button: &Button) -> Option<Action<Operation>> {
        if let Some(action) = self.list.handle_navigation(button) {
            return Some(action);
        }

        match button {
            Button::A => {
                if let Some(item) = self.list.current_item() {
                    Some(Action::Complete(Operation::SelectItem(item.clone())))
                } else {
                    Some(Action::Continue)
                }
            }
            _ => Some(Action::Continue),
        }
    }

    fn draw(&self, screen: &mut Screen) {
        self.list.draw(screen, false, false, |item| item.clone());
    }
}
