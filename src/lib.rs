pub mod engine;
pub mod relay;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::engine;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn unshuffled_match() {
        let mut gs = engine::state::GameState::default();
        while !gs.deck_is_empty() {
            let event = engine::turn(&mut gs);
            assert_eq!(event.winner.is_none(), true);
        }
    }
}
