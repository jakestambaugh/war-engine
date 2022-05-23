use axum::extract::ws::{Message, WebSocket};

use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::{broadcast, mpsc, Mutex},
    task::JoinHandle,
};

use crate::engine::{
    input::GameInputEvent,
    log::GameLogEvent,
    state::{init_gamestate, GameState},
    turn,
};

#[derive(PartialEq, Eq, Hash)]
pub enum PlayerId {
    A,
    B,
}

struct PlayerConnection {
    rx: Arc<SplitStream<WebSocket>>,
    tx: Arc<SplitSink<WebSocket, Message>>,
}

impl PlayerConnection {
    fn new(
        sender: Arc<SplitSink<WebSocket, Message>>,
        receiver: Arc<SplitStream<WebSocket>>,
    ) -> Self {
        Self {
            rx: receiver,
            tx: sender,
        }
    }
}

pub struct RelayServer {
    player_connections: HashMap<PlayerId, PlayerConnection>,
    input_tx_handle: mpsc::UnboundedSender<GameInputEvent>,
    broadcast_tx_handle: broadcast::Sender<GameLogEvent>,

    // This is the task that runs the relay server logic
    task: JoinHandle<()>,
}

impl RelayServer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (btx, _) = broadcast::channel(32);
        let game_state = init_gamestate();
        Self {
            player_connections: HashMap::new(),
            input_tx_handle: tx,
            broadcast_tx_handle: btx.clone(),
            task: tokio::spawn(run_server(rx, btx.clone(), game_state)),
        }
    }

    fn assign_player(&mut self, conn: PlayerConnection) -> &PlayerId {
        if self.player_connections.contains_key(&PlayerId::B) {
            // We already have two players, this shouldn't happen
            panic!()
        } else if self.player_connections.contains_key(&PlayerId::A) {
            self.player_connections.insert(PlayerId::B, conn);
            &PlayerId::B
        } else {
            self.player_connections.insert(PlayerId::A, conn);
            &PlayerId::A
        }
    }

    fn run_player(&mut self, conn: PlayerConnection) {
        let id = self.assign_player(conn);
        // This is janky. We know that we just put the connection in the map, we shouldn't have to do this.
        // Maybe the order of function calls should change? Move the conn into the HashMap after starting the tasks?
        // let conn = self.player_connections.get(id).unwrap();

        tokio::spawn(ws_read(mut conn.rx, self.input_tx_handle));
        tokio::spawn(ws_write(mut conn.tx, self.broadcast_tx_handle.subscribe()));
    }
}

/// Instead of defining a struct, we can define a single function that takes all of the params the struct would hold.
/// This is effectively the "inside-task" half of the relay server
async fn run_server(
    mut input_events: mpsc::UnboundedReceiver<GameInputEvent>,
    output_events: broadcast::Sender<GameLogEvent>,
    mut state: GameState,
) {
    let _input = input_events.recv().await;
    let event = turn(&mut state);
    output_events.send(event).unwrap();
}

/// When a new player requests a websocket connection, we create two tasks: one to read and one to write. They each need a channel
pub async fn handle_socket(socket: WebSocket, relay: Arc<Mutex<RelayServer>>) {
    let (sender, receiver) = socket.split();

    let connection = PlayerConnection::new(Arc::new(sender), Arc::new(receiver));

    // Assign to player
    relay.lock().await.run_player(connection)
}

async fn ws_write(
    sender: &mut SplitSink<WebSocket, Message>,
    mut channel_rx: broadcast::Receiver<GameLogEvent>,
) {
    while let Ok(event) = channel_rx.recv().await {
        if let Err(e) = sender.send(Message::Text(event.description)).await {
            eprintln!("Error while sending websocket message: {:?}", e);
        }
    }
}

async fn ws_read(
    receiver: &mut SplitStream<WebSocket>,
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
