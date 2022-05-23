use super::input::GameInputEvent;
use super::state::GameState;

pub fn make_state_machine() -> &'static str {
    "Hello I'm a state machine"
}

fn generate_next_state(state: &mut GameState, a_input: &GameInputEvent, b_input: &GameInputEvent) {}
