#[derive(Debug, Clone)]
pub struct GameInputEvent {
    _text: String,
}

impl GameInputEvent {
    pub fn new(text: &str) -> Self {
        Self {
            _text: text.to_string(),
        }
    }
}
