extern crate engine;

use engine::game::{turn, GameState, Winner};
use rand;

pub fn main() {
    let mut rng = rand::thread_rng();

    let mut game_state = GameState::default();
    game_state.shuffle(&mut rng);
    while !game_state.deck_is_empty() {
        let winner = turn(&mut game_state);
        match winner {
            Some(Winner::A) => println!("Winner: A"),
            Some(Winner::B) => println!("Winner: B"),
            None => println!("Game over"),
        }
    }
}
