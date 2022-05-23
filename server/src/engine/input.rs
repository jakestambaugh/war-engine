#[derive(Debug, Clone)]
pub struct GameInputEvent {
    text: String,
}

impl GameInputEvent {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}
