use axum::{
    extract::ws::WebSocketUpgrade,
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

use war::relay::{handle_socket, RelayServer};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let relay_server = Arc::new(Mutex::new(RelayServer::new()));

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/connect", get(ws_upgrade_handler))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(relay_server));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
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
    Extension(relay): Extension<Arc<Mutex<RelayServer>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, relay))
}
