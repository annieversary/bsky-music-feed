use std::sync::Arc;

use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

use crate::{
    firehose::{self, Handler, OnPostCreateParams, OnPostDeleteParams},
    link_finder::get_music_links,
    models::{links, posts},
};

pub async fn start_ingest(pool: Pool<Sqlite>) -> Result<()> {
    firehose::listen(Handler::<AppData> {
        on_post_create: Arc::new(move |params, data| Box::pin(on_post_create(params, data))),
        on_post_delete: Arc::new(move |params, data| Box::pin(on_post_delete(params, data))),
        data: Arc::new(AppData { pool }),
    })
    .await
    .context("failed while listening to firehose")?;

    Ok(())
}

struct AppData {
    pool: Pool<Sqlite>,
}

async fn on_post_create(params: OnPostCreateParams<'_>, data: Arc<AppData>) {
    let links = get_music_links(&params.post.text);

    if !links.is_empty() {
        // store post in posts table
        let cid = params.cid.0.to_string();
        if let Err(err) = posts::Post::create(&data.pool, &params.uri, cid).await {
            println!("{err}");
        }

        // store links in links table
        for link in &links {
            if let Err(err) = links::Link::create(&data.pool, link).await {
                println!("{err}");
            }
        }
    }
}

async fn on_post_delete(params: OnPostDeleteParams<'_>, data: Arc<AppData>) {
    // delete post by uri from the db
    if let Err(err) = posts::Post::delete(&data.pool, &params.uri).await {
        println!("{err}");
    }
}
