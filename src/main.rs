use std::sync::Arc;

use anyhow::Context;
use server::start_server;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

use crate::firehose::{Handler, OnPostCreateParams, OnPostDeleteParams};

mod firehose;
mod link_finder;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().context("failed to load .env")?;

    let db_connection_str = std::env::var("DATABASE_URL").context("failed to get db url")?;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_connection_str)
        .await
        .context("failed to connect to db")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run migrations")?;

    firehose::listen(Handler::<AppData> {
        on_post_create: Arc::new(move |params, data| Box::pin(on_post_create(params, data))),
        on_post_delete: Arc::new(move |params, data| Box::pin(on_post_delete(params, data))),
        data: Arc::new(AppData { pool }),
    })
    .await?;

    let server_config = server::Config {
        service_did: "test".to_string(),
        hostname: "test".to_string(),
    };

    start_server(server_config).await;

    Ok(())
}

struct AppData {
    pool: Pool<Sqlite>,
}

async fn on_post_create(params: OnPostCreateParams<'_>, data: Arc<AppData>) {
    let links = link_finder::get_music_links(&params.post.text);

    if !links.is_empty() {
        // store post in posts table
        let cid = params.cid.0.to_string();
        if let Err(err) = models::posts::Post::create(&data.pool, &params.uri, cid).await {
            println!("{err}");
        }

        // store links in links table
        for link in &links {
            if let Err(err) = models::links::Link::create(&data.pool, link).await {
                println!("{err}");
            }
        }
    }
}

async fn on_post_delete(params: OnPostDeleteParams<'_>, data: Arc<AppData>) {
    // delete post by uri from the db
    if let Err(err) = models::posts::Post::delete(&data.pool, &params.uri).await {
        println!("{err}");
    }
}

mod server {
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
}
