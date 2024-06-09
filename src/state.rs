use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use time::OffsetDateTime;

#[cfg(feature = "ssr")]
use {
    axum::extract::FromRef,
    std::sync::{Arc, Mutex},
    tokio::sync::broadcast,
};

pub type SensorStateMap = BTreeMap<i32, SensorData>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SensorData {
    pub name: String,
    pub state: bool,
    pub last_update: OffsetDateTime,
}

impl SensorData {
    pub fn new(name: &str, state: bool) -> Self {
        Self {
            name: name.to_string(),
            state,
            last_update: OffsetDateTime::now_utc(),
        }
    }

    pub fn update_state(&mut self, state: bool) {
        self.state = state;
        self.last_update = OffsetDateTime::now_utc();
    }
}

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
                (0, SensorData::new("Sensor 0", false)),
                (1, SensorData::new("Sensor 1", false)),
                (2, SensorData::new("Sensor 2", false)),
            ]))),
        }
    }
}
