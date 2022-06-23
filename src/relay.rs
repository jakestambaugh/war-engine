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

/// This is the data about the server that lives inside the async RelayServer task. It includes
/// metadata about the server, mostly related to player connection and identification info.
struct ServerState {
    /// When a new relay is created, this field is filled with all of the player ids that the
    /// server could support. Whenever a new player connects to the server, one player id is removed
    /// from the pool and assigned to a player task.
    available_player_slots: Vec<PlayerId>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            available_player_slots: vec![PlayerId::A, PlayerId::B],
        }
    }

    pub fn claim_player_slot(&mut self) -> Option<PlayerId> {
        self.available_player_slots.pop()
    }
}

pub struct RelayServer {
    /// This channel is used to send GameInputEvents from the `ws_read` tasks (incoming websocket messages) to the central
    /// server and state machine. When a new RelayServer is constructed, the rx half of this channel is sent to the `run_server` task
    input_tx_handle: mpsc::UnboundedSender<GameInputEvent>,

    /// When the central server and state machine have a new event that needs to be sent to the clients, it is sent over this channel.
    /// Whenever a new `ws_write` task is created, it subscribes to this channel, creating another rx handle.
    broadcast_tx_handle: broadcast::Sender<GameLogEvent>,

    server_state: Arc<Mutex<ServerState>>,

    // This is the task that runs the relay server logic
    _task: JoinHandle<()>,
}

impl RelayServer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (btx, _) = broadcast::channel(32);
        let game_log = GameLog::new(btx.clone());
        let game_state = init_gamestate(game_log);
        let server_state = Arc::new(Mutex::new(ServerState::new()));
        Self {
            input_tx_handle: tx,
            broadcast_tx_handle: btx.clone(),
            server_state: server_state.clone(),
            _task: tokio::spawn(run_server(rx, btx, game_state, server_state.clone())),
        }
    }

    async fn claim_player_slot(&mut self) -> Option<PlayerId> {
        self.server_state.lock().await.claim_player_slot()
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
    mut game_state: GameState,
    server_state: Arc<Mutex<ServerState>>,
) {
    while !game_state.deck_is_empty() {
        if let Some(_input) = input_events.recv().await {
            if _input._close {
                // This is effectively "disconnecting" this player and returning their id to the pool of available ids
                server_state
                    .lock()
                    .await
                    .available_player_slots
                    .push(_input.player);
            }
            // TODO: figure out how to communicate back to the owning server. This might be impossible, so player count may have to live elsewhere.
            turn(&mut game_state);
        } else {
            tracing::warn!("Got a None input event from the input_events channel");
        }
    }
}

/// When a new player requests a websocket connection, we create two tasks: one to read and one to write. They each need a channel
pub async fn handle_socket(socket: WebSocket, relay: Arc<Mutex<RelayServer>>) {
    // Assign to player
    let mut r = relay.lock().await;
    match r.claim_player_slot().await {
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
