use leptos::*;
use std::collections::BTreeMap;

#[cfg(feature = "ssr")]
use {
    axum::extract::FromRef,
    std::sync::{Arc, Mutex},
    tokio::sync::broadcast,
};

pub type SensorStateMap = BTreeMap<i32, Option<bool>>;

#[cfg(feature = "ssr")]
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub tx: broadcast::Sender<String>,
    pub sensor_state: Arc<Mutex<SensorStateMap>>,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub fn new(leptos_options: LeptosOptions) -> Self {
        let (tx, _) = broadcast::channel(32);
        Self {
            leptos_options,
            tx,
            sensor_state: Arc::new(Mutex::new(SensorStateMap::from([
                (0, None),
                (1, None),
                (2, None),
            ]))),
        }
    }
}
