use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

pub struct Config {
    pub service_did: String,
    pub hostname: String,
}

struct AppState {
    config: Config,
}

pub async fn start_server(config: Config) {
    let app = Router::new()
        .route("/.well-known/did.json", get(well_known))
        .with_state(Arc::new(AppState { config }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn well_known(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(json!({
        "@context": ["https://www.w3.org/ns/did/v1"],
        "id": state.config.service_did,
        "service": [
            {
                "id": "#bsky_fg",
                "type": "BskyFeedGenerator",
                "serviceEndpoint": format!("https://{}", state.config.hostname)
            }
        ]
    }))
}
