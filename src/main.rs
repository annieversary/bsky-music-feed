use anyhow::Context;
use ingest::start_ingest;
use server::{start_server, Config};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

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

    start_server(app_state).await;

    Ok(())
}

mod algos {
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
}

mod atproto {
    #[derive(Debug, Clone)]
    pub struct AtUri<'a> {
        pub did: &'a str,
        pub collection: &'a str,
        pub rkey: &'a str,
    }

    impl<'a> AtUri<'a> {
        pub fn from_str(s: &'a str) -> Result<AtUri<'a>, &'static str> {
            let parts = s
                .strip_prefix("at://")
                .ok_or(r#"record uri must start with "at://""#)?
                .splitn(3, '/')
                .collect::<Vec<_>>();

            if !parts[0].starts_with("did:plc:") {
                return Err(r#"record uri must start with "at://did:plc:""#);
            }

            Ok(Self {
                did: parts[0],
                collection: parts[1],
                rkey: parts[2],
            })
        }
    }

    impl std::fmt::Display for AtUri<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "at://{}/{}/{}", self.did, self.collection, self.rkey)
        }
    }
}
