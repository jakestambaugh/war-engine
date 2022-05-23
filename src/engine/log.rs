use serde::Serialize;
use serde_json::json;
use std::fmt::Debug;
use tokio::sync::broadcast;

use crate::types::card::*;

use super::Outcome;

#[derive(Debug, Serialize, Clone, Copy)]
pub enum PlayerId {
    A,
    B,
}

#[derive(Serialize, Clone, Copy)]
pub struct Comparison {
    pub a_card: Card,
    pub b_card: Card,
}

#[derive(Serialize, Clone)]
pub struct Wager {
    pub player: PlayerId,
    pub cards: Vec<Card>,
}

#[derive(Serialize, Clone)]
pub enum GameLogEvent {
    GameStarted,
    DrewCard(PlayerId, Card),
    ComparedMatch(Comparison),
    ResolvedMatch(Outcome),
    WageredVisible(Wager),
    WageredHidden(Wager),
    ClaimedWager(Wager),
    GameEndedInWar,
    GameEnded,
}

impl Debug for GameLogEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(self))
    }
}

pub struct GameLog {
    logs: Vec<GameLogEvent>,
    tx: broadcast::Sender<GameLogEvent>,
}

impl GameLog {
    pub fn new(tx: broadcast::Sender<GameLogEvent>) -> Self {
        Self {
            logs: Vec::new(),
            tx,
        }
    }

    pub fn log(&mut self, event: GameLogEvent) {
        self.tx.send(event.clone()).unwrap();
        self.logs.push(event);
    }
}
