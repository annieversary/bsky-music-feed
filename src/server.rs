use std::sync::Arc;

use atrium_api::app::bsky::feed::get_feed_skeleton::{OutputData, ParametersData};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;

use crate::{algos::feed, atproto::AtUri, AppState};

pub struct Config {
    pub service_did: String,
    pub publisher_did: String,
    pub hostname: String,
}

pub async fn start_server(app_state: AppState) {
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
        .with_state(Arc::new(app_state));

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
        .map(|rkey| {
            json!({
                "uri": AtUri {
                    did: &state.config.publisher_did,
                    collection: "app.bsky.feed.generator",
                    rkey
                }.to_string()
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

async fn get_feed_skeleton(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ParametersData>,
) -> Result<Json<OutputData>, (StatusCode, &'static str)> {
    let Ok(uri) = AtUri::from_str(&params.feed) else {
        return Err((StatusCode::BAD_REQUEST, "Could not parse feed"));
    };

    if uri.did != state.config.publisher_did || uri.collection != "app.bsky.feed.generator" {
        return Err((StatusCode::BAD_REQUEST, "Usupported algorithm"));
    }

    let output = feed(uri.rkey, &state, &params).await?;

    Ok(Json(output))
}
