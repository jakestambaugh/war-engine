extern crate engine;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use engine::game::{turn, GameState, Winner};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use rand;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/connect", get(ws_upgrade_handler))
        .layer(TraceLayer::new_for_http());

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
async fn root() -> &'static str {
    "Hello, World!"
}

async fn ws_upgrade_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let (sender, receiver) = socket.split();

    let (tx, rx) = unbounded_channel();

    tokio::spawn(ws_write(sender, rx));
    tokio::spawn(ws_read(receiver, tx));
}

async fn ws_read(mut receiver: SplitStream<WebSocket>, channel_tx: UnboundedSender<Message>) {
    while let Some(Ok(msg)) = receiver.next().await {
        if let Err(e) = channel_tx.send(msg) {
            eprintln!("Failed to send to channel: {:?}", e);
        }
    }
}

async fn ws_write(
    mut sender: SplitSink<WebSocket, Message>,
    mut channel_rx: UnboundedReceiver<Message>,
) {
    while let Some(msg) = channel_rx.recv().await {
        if let Err(e) = sender.send(msg).await {
            eprintln!("Error while sending websocket message: {:?}", e);
        }
    }
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn test_random() {
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
}
