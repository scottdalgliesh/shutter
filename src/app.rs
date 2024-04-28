use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[cfg(feature = "ssr")]
use crate::state::AppState;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/shutter.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
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
    }
}

#[component]
fn HomePage() -> impl IntoView {
    use leptos_use::{use_websocket, UseWebsocketReturn};
    let (state, set_state) = create_signal("Un-initialized".to_string());
    let (history, set_history) = create_signal(vec![]);
    let UseWebsocketReturn { message, send, .. } = use_websocket("ws://localhost:3000/ws");

    create_effect(move |_| {
        if let Some(msg) = message.get() {
            set_history.update(|history| history.push(format!("Received: {msg}")));
            send(&msg);
            set_history.update(|history| history.push(format!("Sent: {msg}")));
            set_state.set(msg);
        }
    });

    view! {
        <h1>"Websocket Test"</h1>
        <p>
            "This page demonstrates using a websocket to perform live updates in the UI in response to activity on the server."
        </p>
        <p>
            "Test the websocket connection by either using the \"send request\" button below, or using an external post request to http://127.0.0.1:3000/api/toggle_state"
        </p>
        <p>"Current value: " {state}</p>
        <button on:click=move |_| {
            spawn_local(async {
                toggle_state().await.unwrap();
            })
        }>"Send Request"</button>
        <h2>"Websocket History"</h2>
        <For each=move || history.get().into_iter().enumerate() key=|(index, _)| *index let:item>
            <p>{item.1}</p>
        </For>
    }
}

#[server(ToggleState, "/api", "Url", "toggle_state")]
pub async fn toggle_state() -> Result<(), ServerFnError> {
    let app_state = use_context::<AppState>().unwrap();
    let mut sensor_state = app_state.sensor_state.lock().unwrap();
    *sensor_state = !*sensor_state;
    let tx = app_state.tx.clone();
    tx.send(*sensor_state).unwrap();
    Ok(())
}