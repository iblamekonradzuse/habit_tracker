use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub description: String,
    pub completed: bool,
}

impl Todo {
    pub fn new(description: String) -> Self {
        Todo {
            description,
            completed: false,
        }
    }

    pub fn toggle_completion(&mut self) {
        self.completed = !self.completed;
    }
}
