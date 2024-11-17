use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{Executor, Sqlite};

#[allow(dead_code)]
pub struct Post {
    /// The full uri. Eg: `at://did:plc:asdfghjkl/app.bsky.feed.post/qwertyuiop`
    pub uri: String,
    /// The record CID
    pub cid: String,
    /// The time this post was indexed at
    pub indexed_at: DateTime<Utc>,
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

    pub async fn get_all<'e, E>(executor: E, limit: u8) -> Result<Vec<Post>>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let posts = sqlx::query!(
            "select * from posts order by indexed_at desc, cid desc limit ?",
            limit
        )
        .fetch_all(executor)
        .await?
        .into_iter()
        .filter_map(|post| {
            Some(Post {
                uri: post.uri?,
                cid: post.cid,
                indexed_at: post.indexed_at.and_utc(),
            })
        })
        .collect::<Vec<_>>();

        Ok(posts)
    }

    pub async fn get_all_where_time_under<'e, E>(
        executor: E,
        limit: u8,
        time: DateTime<Utc>,
    ) -> Result<Vec<Post>>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let posts = sqlx::query!(
            "select * from posts where indexed_at < ? order by indexed_at desc, cid desc limit ?",
            time,
            limit
        )
        .fetch_all(executor)
        .await?
        .into_iter()
        .filter_map(|post| {
            Some(Post {
                uri: post.uri?,
                cid: post.cid,
                indexed_at: post.indexed_at.and_utc(),
            })
        })
        .collect::<Vec<_>>();

        Ok(posts)
    }
}
