use super::log::PlayerId;

#[derive(Debug, Clone)]
pub struct GameInputEvent {
    _text: String,
    pub _close: bool,
    pub player: PlayerId,
}

impl GameInputEvent {
    pub fn new(text: &str, source: PlayerId) -> Self {
        Self {
            _text: text.to_string(),
            _close: false,
            player: source,
        }
    }

    pub fn player_disconnect(source: PlayerId) -> Self {
        tracing::debug!("Player disconnected {:?}", &source);
        Self {
            _text: "player disconnected".into(),
            _close: true,
            player: source,
        }
    }
}

// TODO: convert this to an enum
