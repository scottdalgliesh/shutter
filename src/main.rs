#![cfg(feature = "ssr")]

use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    extract::{FromRef, State},
    routing::{get, post},
    response::Response,
    Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use shutter::app::*;
use shutter::fileserv::file_and_error_handler;
use tokio::sync::broadcast;
use futures::{stream::StreamExt, SinkExt};
use std::sync::{Arc, Mutex};

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    leptos_options: LeptosOptions,
    tx: broadcast::Sender<bool>,
    sensor_state: Arc<Mutex<bool>>,
}

impl AppState {
    fn new(leptos_options: LeptosOptions) -> Self {
        let (tx, _) = broadcast::channel(32);
        Self{
            leptos_options,
            tx,
            sensor_state: Arc::new(Mutex::new(false)),
        }
    }
}

#[tokio::main]
async fn main() {
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app_state = AppState::new(leptos_options);
    let app = Router::new()
        .leptos_routes(
            &app_state,
            routes,
            App,
        )
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .route("/toggle_state", get(toggle_state))
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

async fn toggle_state(State(state): State<AppState>) {
    let mut sensor_state = state.sensor_state.lock().unwrap();
    *sensor_state = !*sensor_state;
    let tx = state.tx.clone();
    tx.send(*sensor_state).unwrap();
    logging::log!("Server: updated state to {sensor_state}");
}