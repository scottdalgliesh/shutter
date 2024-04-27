#![cfg(feature = "ssr")]

use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    routing::{get, post},
    response::{IntoResponse, Response},
    Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use shutter::app::*;
use shutter::fileserv::file_and_error_handler;
use tokio::sync::broadcast;


#[tokio::main]
async fn main() {
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let (tx, _rx) = broadcast::channel(32);
    let tx1 = tx.clone();
    let tx2 = tx.clone();

    // build our application with a route
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, App)
        // .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .route("/toggle_state", get(|| toggle_state(tx2)))
        .route("/ws", get(handler))
        // .route("/ws", get(|ws| handler(ws, tx1)))
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    use std::time::Duration;
    logging::log!("Opened websocket on server");    
    for iter in 0..10 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        socket.send(Message::Text(iter.to_string())).await.unwrap();
        logging::log!("Server: sent message {}", iter.to_string());
        if let Ok(Message::Text(msg)) = socket.recv().await.unwrap() {
            logging::log!("Client: received {msg}");
        };
    }
}

// pub async fn handler(ws: WebSocketUpgrade, tx: broadcast::Sender<bool>) -> Response {
//     ws.on_upgrade(|ws| handle_socket(ws, tx))
// }

// async fn handle_socket(mut socket: WebSocket, tx: broadcast::Sender<bool>) {
//     let mut rx = tx.subscribe();
//     if let Ok(state) = rx.recv().await {
//         socket.send(Message::Text(state.to_string())).await.unwrap();
//     }
// }

pub async fn toggle_state(tx: broadcast::Sender<bool>) {
    tx.send(true).unwrap();
    logging::log!("message sent");
}

// pub async fn handler(ws: WebSocketUpgrade, tx: mpsc::Sender<mpsc::Receiver<bool>>) -> Response {
//     ws.on_upgrade(|ws| handle_socket(ws, tx))
// }

// async fn handle_socket(mut socket: WebSocket, tx: mpsc::Sender<mpsc::Receiver<bool>>) {
    
// }