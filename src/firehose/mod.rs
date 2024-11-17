use anyhow::Result;

pub use self::handler::{Handler, OnPostCreateParams, OnPostDeleteParams};

mod handler;
mod stream;
mod subscription;

pub async fn listen<DATA: Send + Sync + 'static>(
    handler: Handler<DATA>,
) -> Result<(), Box<dyn std::error::Error>> {
    subscription::RepoSubscription::new("bsky.network")
        .await?
        .run(handler)
        .await
}
