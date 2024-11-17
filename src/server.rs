use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;
use sqlx::{Pool, Sqlite};

pub struct Config {
    pub service_did: String,
    pub publisher_did: String,
    pub hostname: String,
}

struct AppState {
    config: Config,
    pool: Pool<Sqlite>,
}

pub async fn start_server(config: Config, pool: Pool<Sqlite>) {
    let app = Router::new()
        .route("/.well-known/did.json", get(well_known))
        .route(
            "/xrpc/app.bsky.feed.describeFeedGenerator",
            get(describe_feed_generator),
        )
        .route(
            "/xrpc/app.bsky.feed.getFeedSkeleton",
            get(get_feed_skeleton),
        )
        .with_state(Arc::new(AppState { config, pool }));

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

async fn describe_feed_generator(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let feeds = crate::algos::list()
        .iter()
        .map(|shortname| {
            json!({
                "uri": format!(
                    "at://{}/app.bsky.feed.generator/{}",
                    state.config.publisher_did, shortname
                )
            })
        })
        .collect::<Vec<_>>();

    Json(json!({
        "encoding": "application/json",
        "body": {
            "did": state.config.service_did,
            "feeds": feeds,
        }
    }))
}

async fn get_feed_skeleton(State(state): State<Arc<AppState>>) {
    //
}
