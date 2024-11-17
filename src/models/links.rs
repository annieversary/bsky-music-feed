use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Executor, Sqlite};

use crate::link_finder::{FoundLink, Kind, Site};

#[allow(dead_code)]
pub struct Link {
    url: String,
    kind: Kind,
    site: Site,
    created_at: Utc,
    count: i64,
}

impl Link {
    pub async fn create<'e, E>(executor: E, link: &FoundLink<'_>) -> Result<()>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let now = Utc::now();
        let _ = sqlx::query!(
                "insert into links (url, kind, site, created_at) values (?, ?, ?, ?) on conflict(url) do update set count = count + 1",
                link.url,
                link.kind,
                link.site,
                now,
            )
                .execute(executor)
                .await
                .context("failed to create link")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use sqlx::{Connection, SqliteConnection};

    async fn conn() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();

        sqlx::migrate!("./migrations").run(&mut conn).await.unwrap();

        conn
    }

    #[tokio::test]
    async fn test_count_defaults_to_1() {
        let mut conn = conn().await;

        Link::create(
            &mut conn,
            &FoundLink {
                url: "test",
                kind: Kind::Track,
                site: Site::Bandcamp,
            },
        )
        .await
        .unwrap();

        let count = sqlx::query_scalar!("select count from links")
            .fetch_one(&mut conn)
            .await
            .unwrap();

        assert_eq!(1, count);
    }

    #[tokio::test]
    async fn test_count_increases_if_duplicate_url() {
        let mut conn = conn().await;

        Link::create(
            &mut conn,
            &FoundLink {
                url: "test",
                kind: Kind::Track,
                site: Site::Bandcamp,
            },
        )
        .await
        .unwrap();

        Link::create(
            &mut conn,
            &FoundLink {
                url: "test",
                kind: Kind::Album,
                site: Site::Spotify,
            },
        )
        .await
        .unwrap();

        let count = sqlx::query_scalar!("select count from links where url = 'test'")
            .fetch_one(&mut conn)
            .await
            .unwrap();

        assert_eq!(2, count);
    }

    #[tokio::test]
    async fn test_count_doesnt_increase_if_url_is_different() {
        let mut conn = conn().await;

        Link::create(
            &mut conn,
            &FoundLink {
                url: "test",
                kind: Kind::Track,
                site: Site::Bandcamp,
            },
        )
        .await
        .unwrap();

        Link::create(
            &mut conn,
            &FoundLink {
                url: "other",
                kind: Kind::Track,
                site: Site::Bandcamp,
            },
        )
        .await
        .unwrap();

        let count = sqlx::query_scalar!("select count from links where url = 'test'")
            .fetch_one(&mut conn)
            .await
            .unwrap();

        assert_eq!(1, count);
    }
}
