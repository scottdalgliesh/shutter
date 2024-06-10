use crate::error_template::{AppError, ErrorTemplate};
use crate::state::{SensorData, SensorStateMap};
use leptonic::components::prelude::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::use_interval_fn;

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
    let sensor_view = move || {
        state.get().map(move |state| match state {
            Ok(inner) => {
                if inner.is_empty() {
                    view! { <p>"No sensors were found."</p> }.into_view()
                } else {
                    inner
                        .into_values()
                        .map(move |data| {
                            view! { <SensorCard data=data/> }
                        })
                        .collect_view()
                }
            }
            Err(_) => view! { <p>"Error loading data from Server."</p> }.into_view(),
        })
    };

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

                <Transition fallback=move || {
                    view! { <p>"Loading..."</p> }
                }>{sensor_view}</Transition>

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
fn sensor_card(data: SensorData) -> impl IntoView {
    // set up reactive current time (updated once per second)
    let (now, set_now) = create_signal(time::OffsetDateTime::now_utc());
    use_interval_fn(move || set_now.set(time::OffsetDateTime::now_utc()), 1_000);
    let since_last_update = move || now.get() - data.last_update;
    let is_active = move || since_last_update() < time::Duration::seconds(10);

    let color = move || match (data.state, is_active()) {
        (true, true) => "cornflowerblue",
        (false, true) => "coral",
        (_, false) => "grey",
    };

    view! {
        <div class="sensor_card" style:background-color=color>
            {data.name}
        </div>
    }
}

#[server]
pub async fn get_sensors() -> Result<SensorStateMap, ServerFnError> {
    Ok(expect_context::<AppState>()
        .sensor_state
        .lock()
        .unwrap()
        .clone())
}
