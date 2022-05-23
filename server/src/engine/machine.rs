use super::{state::GameState, GameInput};

pub fn make_state_machine() -> &'static str {
    "Hello I'm a state machine"
}

struct InputCommand {
    text: String,
}

fn generate_next_state(state: &mut GameState, a_input: &InputCommand, b_input: &InputCommand) {}
