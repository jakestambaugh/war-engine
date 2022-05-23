use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug, Serialize)]
pub enum Winner {
    A,
    B,
}

#[derive(Serialize)]
pub struct GameLogEvent {
    pub description: String,
    pub winner: Option<Winner>,
}

impl GameLogEvent {
    pub fn new() -> Self {
        Self {
            description: String::new(),
            winner: None,
        }
    }

    pub fn append(&mut self, s: &str) {
        self.description.push_str(s);
        self.description.push('\n');
    }

    pub fn a_wins(&mut self) {
        self.winner = Some(Winner::A)
    }

    pub fn b_wins(&mut self) {
        self.winner = Some(Winner::B)
    }
}

impl Debug for GameLogEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.description.lines().next().unwrap_or("No events")
        )
    }
}

impl Default for GameLogEvent {
    fn default() -> Self {
        Self::new()
    }
}
