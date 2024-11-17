use std::sync::Arc;
use firehose::Handler;
use crate::firehose::{OnPostCreateParams, OnPostDeleteParams};

mod firehose;
mod link_finder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    firehose::listen(Handler::<AppData> {
        on_post_create: Arc::new(move |params, data| Box::pin(on_post_create(params, data))),
        on_post_delete: Arc::new(move |params, data| Box::pin(on_post_delete(params, data))),
        data: Arc::new(AppData {}),
    })
    .await?;

    Ok(())
}

struct AppData {
}

async fn on_post_create(params: OnPostCreateParams<'_>, data: Arc<AppData>) {
    let links = link_finder::get_music_links(&params.post.text);

    if !links.is_empty() {
        // TODO store post in posts table
        // sqlx::query!("insert into posts (uri) values (?)", params.uri);

        // TODO store links in links table
        for link in &links {
            // sqlx::query!("insert into links (url, kind, site) values (?, ?, ?)", link.url, link.kind, link.site);
        }
    }
}

async fn on_post_delete(params: OnPostDeleteParams<'_>, data: Arc<AppData>) {
    // TODO delete post by uri from the db
    // sqlx::query!("delete from posts where uri = ?", params.uri);
}
