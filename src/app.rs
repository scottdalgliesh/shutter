use crate::error_template::{AppError, ErrorTemplate};
use crate::state::{SensorData, SensorStateMap};
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::{use_interval_fn, use_timeout_fn, UseTimeoutFnReturn};
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::state::AppState;

#[component]
pub fn app() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/shutter.css"/>
        <Title text="Welcome to Leptos"/>
        <Root default_theme=LeptonicTheme::default()>
            <Router fallback=|| {
                let mut outside_errors = Errors::default();
                outside_errors.insert_with_default_key(AppError::NotFound);
                view! { <ErrorTemplate outside_errors/> }.into_view()
            }>
                <main>
                    <Routes>
                        <Route path="" view=HomePage/>
                    </Routes>
                </main>
            </Router>
        </Root>
    }
}

#[component]
fn home_page() -> impl IntoView {
    let state = create_resource(move || (), move |_| get_sensors());
    let (history, set_history) = create_signal(vec![]);
    use leptos_use::{use_websocket, UseWebsocketReturn};
    let UseWebsocketReturn { message, send, .. } = use_websocket("ws://localhost:3000/ws");

    // flag to pause reactive updates during deletion of a sensor from the database
    let (pause_updates, set_pause_updates) = create_signal(false);

    create_effect(move |_| {
        if let (Some(msg), false) = (message.get(), pause_updates.get()) {
            let msg_state = serde_json::from_str(&msg).unwrap();
            set_history.update(|history| history.push(format!("Received: {msg}")));
            send(&msg);
            state.set(Ok(msg_state));
        }
    });

    view! {
        <div id="app_window">
            <div id="app_content">
                <h1>"Sensor Readout"</h1>
                <Tabs mount=Mount::Once>
                    <Tab name="Sensors" label="Sensors".into_view()>
                        <div id="tab_container">
                        <Transition fallback=move || {view! {<p>"Loading..."</p>}}>
                            <ErrorBoundary fallback=move |_| view! {"Error loading data"}>
                                { state.and_then(|_| ()) } // triggers ErrorBoundary if applicable
                                <For
                                    each= move || match state.get() {
                                        Some(Ok(data)) => data.into_values(),
                                        _ => Default::default(),
                                    }
                                    key= |item| item.id
                                    children= move |item| {
                                        let memo = create_memo(move |_| {
                                            // safe to unwrap here since None/Error cases handled outside of <For>
                                            state.get().unwrap().unwrap().get(&item.id).unwrap().clone()
                                        });
                                        view!{<SensorCard data=memo set_pause_updates/>}
                                    }
                                />
                            </ErrorBoundary>
                        </Transition>
                        </div>
                    </Tab>
                    <Tab name="Info" label="Info".into_view()>
                        <p>
                            "This page demonstrates using a websocket to perform live updates in the UI in response to activity on the server."
                        </p>
                        <p>
                            "Test the websocket connection by using an external post request to http://127.0.0.1:3000/api/<sensor_id>/<sensor_state>"
                        </p>
                    </Tab>
                    <Tab name="Diagnostic Data" label="Diagnostic".into_view()>
                        <div id="tab_container">
                            <h2>"Websocket History"</h2>
                            <For
                                each=move || history.get().into_iter().enumerate()
                                key=|(index, _)| *index
                                let:item
                            >
                                <p>{item.1}</p>
                            </For>
                        </div>
                    </Tab>
                </Tabs>
            </div>
        </div>
    }
}

#[component]
fn sensor_card(data: Memo<SensorData>, set_pause_updates: WriteSignal<bool>) -> impl IntoView {
    // set up reactive current time (updated once per second)
    logging::log!("Rerendering card");
    let (now, set_now) = create_signal(time::OffsetDateTime::now_utc());
    use_interval_fn(move || set_now.set(time::OffsetDateTime::now_utc()), 1_000);
    let since_last_update = move || now.get() - data.get().last_update;
    let is_active = move || since_last_update() < time::Duration::seconds(10);

    let color = move || match (data.get().state, is_active()) {
        (true, true) => "cornflowerblue",
        (false, true) => "coral",
        (_, false) => "grey",
    };

    let (show_config_modal, set_show_config_modal) = create_signal(false);
    let (input, set_input) = create_signal(data.get_untracked().name);
    let disable_update_button = Signal::derive(move || input.get() == data.get().name);

    let toasts = expect_context::<Toasts>();
    let push_toast = move |variant, header: &str, body: &str| {
        toasts.push(Toast {
            id: Uuid::new_v4(),
            created_at: time::OffsetDateTime::now_utc(),
            variant,
            header: header.to_string().into_view(),
            body: body.to_string().into_view(),
            timeout: ToastTimeout::DefaultDelay,
        })
    };

    // warn user if no response received from server
    let UseTimeoutFnReturn {
        start: timeout_start,
        stop: timeout_stop,
        ..
    } = use_timeout_fn(
        move |_: ()| {
            push_toast(
                ToastVariant::Warn,
                "Cannot Reach Server",
                "No response received from server",
            );
        },
        3000.0,
    );
    let timeout_start_clone = timeout_start.clone();
    let timeout_stop_clone = timeout_stop.clone();

    let delete_sensor = create_server_action::<DeleteSensor>();
    let delete_sensor_callback = Callback::new(move |_| {
        timeout_start(());
        set_pause_updates.set(true);
        delete_sensor.dispatch(DeleteSensor { id: data.get().id });
        set_show_config_modal.set(false);
    });
    let update_sensor_name = create_server_action::<UpdateSensorName>();
    let update_sensor_name_callback = Callback::new(move |_| {
        timeout_start_clone(());
        update_sensor_name.dispatch(UpdateSensorName {
            id: data.get().id,
            name: input.get().to_string(),
        });
        set_show_config_modal.set(false);
    });
    let _ = create_effect(move |_| {
        if update_sensor_name.value().get().is_some() {
            timeout_stop_clone();
            push_toast(
                ToastVariant::Success,
                "Sensor Updated",
                "Sensor name updated on server.",
            );
        }
    });
    let _ = create_effect(move |_| {
        if delete_sensor.value().get().is_some() {
            timeout_stop();
            push_toast(
                ToastVariant::Success,
                "Sensor Deleted",
                "Sensor deleted on server.",
            );
            set_pause_updates.set(false);
        }
    });

    view! {
        <div class="sensor_card" style:background-color=color>
            {move || data.get().name}
            <div class="sensor_config"><Button on_press= move |_| set_show_config_modal.set(true) variant=ButtonVariant::Flat><Icon icon=icondata::AiSettingOutlined /></Button></div>
        </div>
        <Modal
            show_when=show_config_modal
            on_escape=move || set_show_config_modal(false)
            on_backdrop_interaction=move || set_show_config_modal(false)
        >
            <ModalHeader><ModalTitle>"Configure Sensor"</ModalTitle></ModalHeader>
            <ModalBody>
                "Update sensor label."
                <TextInput get=input set=set_input/>
            </ModalBody>
            <ModalFooter>
                <ButtonWrapper>
                    <Button on_press=delete_sensor_callback color=ButtonColor::Danger>"Delete"</Button>
                    <Button on_press=update_sensor_name_callback disabled=disable_update_button>"Update"</Button>
                    <Button on_press=move |_| {set_show_config_modal(false); set_input.set(data.get().name)} color=ButtonColor::Secondary>"Cancel"</Button>
                </ButtonWrapper>
            </ModalFooter>
        </Modal>
    }
}

#[server]
pub async fn get_sensors() -> Result<SensorStateMap, ServerFnError> {
    // uncomment to simulate server error
    // Err(ServerFnError::ServerError("error".to_string()))
    Ok(expect_context::<AppState>()
        .sensor_state
        .lock()
        .unwrap()
        .clone())
}

#[server]
pub async fn update_sensor_name(id: i32, name: String) -> Result<(), ServerFnError> {
    let app_state = expect_context::<AppState>();
    let name_ = name.clone();
    let mut sensor_map = app_state.sensor_state.lock().unwrap();
    sensor_map.entry(id).and_modify(|value| value.name = name_);

    let msg = serde_json::to_string(&sensor_map.clone()).unwrap();
    let server_tx = app_state.tx.clone();
    let _ = server_tx.send(msg);
    drop(sensor_map);

    logging::log!("Server: updated sensor: {:?} to name {:?}", id, name);
    Ok(())
}

#[server]
pub async fn delete_sensor(id: i32) -> Result<(), ServerFnError> {
    let app_state = expect_context::<AppState>();
    let mut sensor_map = app_state.sensor_state.lock().unwrap();
    sensor_map.remove(&id);

    let msg = serde_json::to_string(&sensor_map.clone()).unwrap();
    let server_tx = app_state.tx.clone();
    let _ = server_tx.send(msg);
    drop(sensor_map);

    logging::log!("Server: deleted sensor {:?}", id);
    Ok(())
}
