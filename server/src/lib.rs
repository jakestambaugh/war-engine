pub mod card;
pub mod deck;
pub mod game;

#[cfg(test)]
mod tests {
    use crate::game;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn unshuffled_match() {
        let mut gs = game::GameState::default();
        while !gs.deck_is_empty() {
            let event = game::turn(&mut gs);
            assert_eq!(event.winner.is_none(), true);
        }
    }
}
