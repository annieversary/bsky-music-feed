use anyhow::Result;
use atrium_api::{
    app::bsky::feed::{
        defs::SkeletonFeedPostData,
        get_feed_skeleton::{OutputData, ParametersData},
    },
    types::Object,
};
use axum::http::StatusCode;
use chrono::DateTime;

use crate::{models::posts::Post, AppState};

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

    match output {
        Ok(output) => Ok(output),
        Err(_err) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Error")),
    }
}

async fn music(state: &AppState, params: &ParametersData) -> Result<OutputData> {
    // TODO this can go in a function
    let limit = params.limit.map(|limit| limit.into()).unwrap_or(20);
    let cursor = params
        .cursor
        .as_deref()
        .and_then(|time| time.parse::<i64>().ok())
        .and_then(DateTime::from_timestamp_micros);

    // get the recent posts
    let posts = if let Some(time) = cursor {
        Post::get_all_where_time_under(&state.pool, limit, time).await?
    } else {
        Post::get_all(&state.pool, limit).await?
    };

    // update the cursor to be the timestamp of the last post we return
    let cursor = posts
        .last()
        .map(|post| post.indexed_at.timestamp_millis().to_string());

    let feed = posts
        .into_iter()
        .map(|post| {
            Object::from(SkeletonFeedPostData {
                post: post.uri,
                feed_context: None,
                reason: None,
            })
        })
        .collect();

    Ok(OutputData { cursor, feed })
}
