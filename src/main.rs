use anyhow::Context;
use ingest::start_ingest;
use server::{start_server, Config};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

mod algos;
mod atproto;
mod firehose;
mod ingest;
mod link_finder;
mod models;
mod server;

pub struct AppState {
    pub config: Config,
    pub pool: Pool<Sqlite>,
}

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

    tokio::spawn({
        let pool = pool.clone();
        async move {
            start_ingest(pool).await.unwrap();
        }
    });

    let config = server::Config {
        service_did: std::env::var("FEEDGEN_SERVICE_DID")
            .context("failed to get FEEDGEN_SERVICE_DID")?,
        publisher_did: std::env::var("FEEDGEN_PUBLISHER_DID")
            .context("failed to get FEEDGEN_PUBLISHER_DID")?,
        hostname: std::env::var("FEEDGEN_HOSTNAME").context("failed to get FEEDGEN_HOSTNAME")?,
    };

    let app_state = AppState { config, pool };

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|a| a.parse().ok())
        .unwrap_or(3000);

    start_server(app_state, port).await;

    Ok(())
}
