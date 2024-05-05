#![cfg(feature = "ssr")]

use axum::{
    debug_handler, extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State}, response::Response, routing::{get, post}, Router
};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use shutter::app::*;
use shutter::state::AppState;
use shutter::fileserv::file_and_error_handler;
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
        .route("/api/:sensor_id/:sensor_state", post(set_sensor_state))
        .route("/ws", get(socket_handler))
        .fallback(file_and_error_handler)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn socket_handler(ws: WebSocketUpgrade, State(app_state): State<AppState>) -> Response {
    ws.on_upgrade(|ws| socket(ws, app_state))
}

async fn socket(socket: WebSocket, app_state: AppState) {
    let mut server_rx = app_state.tx.subscribe();
    let (mut ws_tx, mut ws_rx) = socket.split();
    logging::log!("Websocket opened on server");

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = server_rx.recv().await {
            if ws_tx.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = ws_rx.next().await {
            logging::log!("Client echo: {text}");
        } 
    });

    // if either task completes, abort the other as well
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

#[debug_handler]
async fn set_sensor_state(Path((sensor_id, sensor_state)): Path<(usize, bool)>, State(app_state): State<AppState>) {
    
    // update sensor state on server    
    let mut app_sensor_state = app_state.sensor_state.lock().unwrap();
    app_sensor_state.0[sensor_id] = Some(sensor_state);

    // serialize sensor state to be passed to websocket
    let msg = serde_json::to_string(&app_sensor_state.clone()).unwrap();
    let server_tx = app_state.tx.clone();
    let _ = server_tx.send(msg); 
    logging::log!("Server: updated sensor id: {:?} to state {:?}", sensor_id, sensor_state);
}