use axum::extract::ws::{Message, WebSocket};

use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use std::sync::Arc;
use tokio::{
    sync::{broadcast, mpsc, Mutex},
    task::JoinHandle,
};

use crate::engine::{
    input::GameInputEvent,
    log::{GameLog, GameLogEvent},
    state::{init_gamestate, GameState},
    turn,
};

#[derive(PartialEq, Eq, Hash)]
pub enum PlayerId {
    A,
    B,
}

pub struct RelayServer {
    input_tx_handle: mpsc::UnboundedSender<GameInputEvent>,
    broadcast_tx_handle: broadcast::Sender<GameLogEvent>,

    // This is the task that runs the relay server logic
    _task: JoinHandle<()>,
}

impl RelayServer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (btx, _) = broadcast::channel(32);
        let game_log = GameLog::new(btx.clone());
        let game_state = init_gamestate(game_log);
        Self {
            input_tx_handle: tx,
            broadcast_tx_handle: btx.clone(),
            _task: tokio::spawn(run_server(rx, btx, game_state)),
        }
    }

    fn run_player(&mut self, socket: WebSocket) {
        let (sender, receiver) = socket.split();

        tokio::spawn(ws_read(receiver, self.input_tx_handle.clone()));
        tokio::spawn(ws_write(sender, self.broadcast_tx_handle.subscribe()));
    }
}

impl Default for RelayServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Instead of defining a struct, we can define a single function that takes all of the params the struct would hold.
/// This is effectively the "inside-task" half of the relay server
async fn run_server(
    mut input_events: mpsc::UnboundedReceiver<GameInputEvent>,
    _output_events: broadcast::Sender<GameLogEvent>,
    mut state: GameState,
) {
    while !state.deck_is_empty() {
        let _input = input_events.recv().await;
        turn(&mut state);
    }
}

/// When a new player requests a websocket connection, we create two tasks: one to read and one to write. They each need a channel
pub async fn handle_socket(socket: WebSocket, relay: Arc<Mutex<RelayServer>>) {
    // Assign to player
    relay.lock().await.run_player(socket)
}

async fn ws_write(
    mut sender: SplitSink<WebSocket, Message>,
    mut channel_rx: broadcast::Receiver<GameLogEvent>,
) {
    while let Ok(event) = channel_rx.recv().await {
        if let Err(e) = sender
            .send(Message::Text(serde_json::to_string(&event).unwrap()))
            .await
        {
            eprintln!("Error while sending websocket message: {:?}", e);
            return;
        }
    }
}

async fn ws_read(
    mut receiver: SplitStream<WebSocket>,
    channel_tx: mpsc::UnboundedSender<GameInputEvent>,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        tracing::debug!("websocket message: {:?}", msg);

        // TODO: translate websocket protocol into game events
        let input = GameInputEvent::new(&msg.into_text().unwrap());

        if let Err(e) = channel_tx.send(input) {
            eprintln!("Failed to send to channel: {:?}", e);
        }
    }
}
