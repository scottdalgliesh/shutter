use leptos::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use {
    axum::extract::FromRef,
    std::sync::{Arc, Mutex},
    tokio::sync::broadcast,
};

#[cfg(feature = "ssr")]
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub tx: broadcast::Sender<String>,
    pub sensor_state: Arc<Mutex<SensorState>>,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub fn new(leptos_options: LeptosOptions) -> Self {
        let (tx, _) = broadcast::channel(32);
        Self {
            leptos_options,
            tx,
            sensor_state: Arc::new(Mutex::new(SensorState::default())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SensorState(pub [Option<bool>; 3]);
