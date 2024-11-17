use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Executor, Sqlite};

#[allow(dead_code)]
pub struct Post {
    /// The full uri. Eg: `at://did:plc:asdfghjkl/app.bsky.feed.post/qwertyuiop`
    uri: String,
    /// The record CID
    cid: String,
    /// The time this post was indexed at
    indexed_at: Utc,
}

impl Post {
    pub async fn create<'e, E>(executor: E, uri: &str, cid: String) -> Result<()>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let now = Utc::now();
        sqlx::query!(
            "insert into posts (uri, cid, indexed_at) values (?, ?, ?) on conflict(uri) do nothing",
            uri,
            cid,
            now,
        )
        .execute(executor)
        .await
        .context("failed to create post")?;

        Ok(())
    }

    pub async fn delete<'e, E>(executor: E, uri: &str) -> Result<()>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        sqlx::query!("delete from posts where uri = ?", uri)
            .execute(executor)
            .await
            .with_context(|| format!("failed to delete post with uri {uri}"))?;

        Ok(())
    }
}
