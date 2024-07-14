use crate::error_template::{AppError, ErrorTemplate};
use crate::state::{SensorData, SensorStateMap};
// use icondata;
use leptonic::components::prelude::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::use_interval_fn;
// use thaw;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::state::AppState;

#[component]
pub fn app() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/shutter.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
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

    create_effect(move |_| {
        if let Some(msg) = message.get() {
            let msg_state = serde_json::from_str(&msg).unwrap();
            set_history.update(|history| history.push(format!("Received: {msg}")));
            send(&msg);
            state.set(Ok(msg_state));
        }
    });

    view! {
        <div id="app_window">
            <div id="app_content">
                <h1>"Websocket Test"</h1>
                <p>
                    "This page demonstrates using a websocket to perform live updates in the UI in response to activity on the server."
                </p>
                <p>
                    "Test the websocket connection by using an external post request to http://127.0.0.1:3000/api/<sensor_id>/<sensor_state>"
                </p>
                <h2>Sensors</h2>
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
                                view!{<SensorCard data=memo/>}
                            }
                        />
                    </ErrorBoundary>
                </Transition>

                <h2>"Websocket History"</h2>
                <For
                    each=move || history.get().into_iter().enumerate()
                    key=|(index, _)| *index
                    let:item
                >
                    <p>{item.1}</p>
                </For>
            </div>
        </div>
    }
}

#[component]
fn sensor_card(data: Memo<SensorData>) -> impl IntoView {
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

    // let show_config_modal = create_rw_signal(false);
    let (show_config_modal, set_show_config_modal) = create_signal(false);
    let (input, set_input) = create_signal(data.get_untracked().name);
    let toasts = expect_context::<Toasts>();

    let update_sensor_name = create_server_action::<UpdateSensorName>();
    let update_sensor_name_callback = Callback::new(move |_| {
        update_sensor_name.dispatch(UpdateSensorName {
            id: data.get().id,
            name: input.get().to_string(),
        });
        set_show_config_modal.set(false);
    });
    let _ = create_effect(move |_| {
        if update_sensor_name.value().get().is_some() {
            toasts.push(Toast {
                id: Uuid::new_v4(),
                created_at: time::OffsetDateTime::now_utc(),
                variant: ToastVariant::Success,
                header: "Sensor Name Updated".into_view(),
                body: "Sensor name successfully updated on server".into_view(),
                timeout: ToastTimeout::DefaultDelay,
            });
            logging::log!("Triggered")
        }
    });

    view! {
        <div class="sensor_card" style:background-color=color>
            {move || data.get().name}
            <div class="sensor_config"><Button on_press= move |_| set_show_config_modal.set(true) variant=ButtonVariant::Flat><Icon icon=icondata::AiSettingOutlined /></Button></div>
            // <div class="sensor_config"><thaw::Button on_click= move |_| set_show_config_modal.set(true) variant=thaw::ButtonVariant::Text><thaw::Icon icon=icondata::AiSettingOutlined /></thaw::Button></div>
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
                    <Button on_press=update_sensor_name_callback >"Update"</Button>
                    <Button on_press=move |_| set_show_config_modal(false) color=ButtonColor::Secondary>"Cancel"</Button>
                </ButtonWrapper>
            </ModalFooter>
        </Modal>

        // <thaw::Modal title="Configure Sensor" show=show_config_modal>
        //     <label for="update_name_input">"Update sensor label."</label>
        //     <thaw::Input value=name attr:id="update_name_input"/>
        //     <thaw::Button on_click= move |_| update_sensor_name.dispatch((id.get(), name.get().clone())) variant=thaw::ButtonVariant::Primary>"Update"</thaw::Button>
        //     <thaw::Button on_click= move |_| show_config_modal.set(false) color=thaw::ButtonColor::Warning>"Cancel"</thaw::Button>
        // </thaw::Modal>

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
