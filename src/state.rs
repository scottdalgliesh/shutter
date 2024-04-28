use axum::extract::FromRef;
use leptos::*;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub tx: broadcast::Sender<bool>,
    pub sensor_state: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new(leptos_options: LeptosOptions) -> Self {
        let (tx, _) = broadcast::channel(32);
        Self {
            leptos_options,
            tx,
            sensor_state: Arc::new(Mutex::new(false)),
        }
    }
}
