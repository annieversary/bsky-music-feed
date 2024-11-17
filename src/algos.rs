use atrium_api::app::bsky::feed::get_feed_skeleton::{OutputData, ParametersData};
use axum::http::StatusCode;

use crate::AppState;

pub fn list() -> &'static [&'static str] {
    &["music", "spotify"]
}

pub async fn feed(
    feed: &str,
    state: &AppState,
    params: &ParametersData,
) -> Result<OutputData, (StatusCode, &'static str)> {
    let output = match feed {
        "music" => music(state, params).await,
        _ => return Err((StatusCode::BAD_REQUEST, "Usupported algorithm")),
    };

    Ok(output)
}

async fn music(state: &AppState, params: &ParametersData) -> OutputData {
    // TODO get all posts

    let cursor = None;
    let feed = vec![];

    OutputData { cursor, feed }
}
