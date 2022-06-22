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
    log::{GameLog, GameLogEvent, PlayerId},
    state::{init_gamestate, GameState},
    turn,
};

pub struct RelayServer {
    input_tx_handle: mpsc::UnboundedSender<GameInputEvent>,
    broadcast_tx_handle: broadcast::Sender<GameLogEvent>,

    /// When a new relay is created, this field is filled with all of the player ids that the
    /// server could support. Whenever a new player connects to the server, one player id is removed
    /// from the pool and assigned to a player task.
    available_player_slots: Vec<PlayerId>,

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
            available_player_slots: vec![PlayerId::A, PlayerId::B],
            _task: tokio::spawn(run_server(rx, btx, game_state)),
        }
    }

    fn claim_player_slot(&mut self) -> Option<PlayerId> {
        self.available_player_slots.pop()
    }

    fn run_player(&mut self, socket: WebSocket, id: PlayerId) {
        let (sender, receiver) = socket.split();

        tokio::spawn(ws_read(receiver, self.input_tx_handle.clone(), id));
        tokio::spawn(ws_write(sender, self.broadcast_tx_handle.subscribe(), id));
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

        // TODO: figure out how to communicate back to the owning server. This might be impossible, so player count may have to live elsewhere.
        turn(&mut state);
    }
}

/// When a new player requests a websocket connection, we create two tasks: one to read and one to write. They each need a channel
pub async fn handle_socket(socket: WebSocket, relay: Arc<Mutex<RelayServer>>) {
    // Assign to player
    let mut r = relay.lock().await;
    match r.claim_player_slot() {
        Some(id) => r.run_player(socket, id),
        None => {
            // Send disconnect message
            let mut socket = socket;
            socket
                .send(Message::Text("Server full".to_owned()))
                .await
                .unwrap();
            socket.close().await.unwrap();
        }
    }
}

async fn ws_write(
    mut sender: SplitSink<WebSocket, Message>,
    mut channel_rx: broadcast::Receiver<GameLogEvent>,
    id: PlayerId,
) {
    // Send a player id at the beginning of the stream
    sender
        .send(Message::Text(format!("Player Id: {:?}", id)))
        .await
        .unwrap();
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
    id: PlayerId,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        tracing::debug!("websocket message: {:?}", msg);

        // TODO: translate websocket protocol into game events
        let input = match msg {
            Message::Close(_) => GameInputEvent::player_disconnect(id),
            _ => GameInputEvent::new(&msg.into_text().unwrap(), id),
        };
        if let Err(e) = channel_tx.send(input) {
            eprintln!("Failed to send to channel: {:?}", e);
        }
    }
}
