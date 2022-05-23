use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

use war::engine::{log::GameLogEvent, state::GameState, turn};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let game_state = Arc::new(Mutex::new(init_gamestate()));

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/connect", get(ws_upgrade_handler))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(game_state));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> Html<&'static str> {
    let main_html = include_str!("../websocket.html");
    Html(main_html)
}

async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<Mutex<GameState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<Mutex<GameState>>) {
    let (sender, receiver) = socket.split();

    let (tx, rx) = unbounded_channel();

    tokio::spawn(ws_write(sender, rx));
    tokio::spawn(ws_read(receiver, tx, state));
}

async fn ws_write(
    mut sender: SplitSink<WebSocket, Message>,
    mut channel_rx: UnboundedReceiver<GameLogEvent>,
) {
    while let Some(event) = channel_rx.recv().await {
        if let Err(e) = sender.send(Message::Text(event.description)).await {
            eprintln!("Error while sending websocket message: {:?}", e);
        }
    }
}

async fn ws_read(
    mut receiver: SplitStream<WebSocket>,
    channel_tx: UnboundedSender<GameLogEvent>,
    state: Arc<Mutex<GameState>>,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        tracing::debug!("websocket message: {:?}", msg);

        let gs = &mut *state.lock().await;
        let event = turn(gs);

        if let Err(e) = channel_tx.send(event) {
            eprintln!("Failed to send to channel: {:?}", e);
        }
    }
}

fn init_gamestate() -> GameState {
    let mut gamestate = GameState::default();
    let mut rng = rand::thread_rng();
    gamestate.shuffle(&mut rng);
    gamestate
}

#[cfg(test)]
mod tests {
    use war::engine::log::Winner;

    use super::*;
    pub fn test_random() {
        let mut rng = rand::thread_rng();

        let mut game_state = GameState::default();
        game_state.shuffle(&mut rng);
        while !game_state.deck_is_empty() {
            let event = turn(&mut game_state);
            match event.winner {
                Some(Winner::A) => println!("Winner: A"),
                Some(Winner::B) => println!("Winner: B"),
                None => println!("Game over"),
            }
        }
    }
}
