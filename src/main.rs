#![cfg(feature = "ssr")]

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State}, response::Response, routing::get, Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use shutter::app::*;
use shutter::state::AppState;
use shutter::fileserv::file_and_error_handler;
use tokio::sync::broadcast;
use futures::{stream::StreamExt, SinkExt};


#[tokio::main]
async fn main() {
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app_state = AppState::new(leptos_options);
    let app = Router::new()
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || provide_context(app_state.clone())
            },
            App,
        )
        .route("/ws", get(socket_handler))
        .fallback(file_and_error_handler)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn socket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|ws| socket(ws, state.tx))
}

async fn socket(socket: WebSocket, tx: broadcast::Sender<bool>) {
    let mut server_rx = tx.subscribe();
    let (mut ws_tx, mut ws_rx) = socket.split();
    logging::log!("Websocket opened on server");

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = server_rx.recv().await {
            if ws_tx.send(Message::Text(msg.to_string())).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = ws_rx.next().await {
            logging::log!("Server received: {text}");
        } 
    });

    // if either task completes, abort the other as well
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}