use std::{future::Future, pin::Pin, sync::Arc};

use anyhow::{anyhow, Result};
use atrium_api::{
    app::bsky::feed::{
        post::{Record, RecordData as PostRecordData},
        Post as AtriumPost,
    },
    com::atproto::sync::subscribe_repos::Commit,
    types::{Collection, Object},
};

use super::subscription::CommitHandler;

pub type Post = Object<PostRecordData>;

#[allow(dead_code)]
pub struct OnPostCreateParams<'a> {
    pub post: &'a Post,
    pub commit: &'a Commit,
    /// The full uri. Eg: `at://did:plc:asdfghjkl/app.bsky.feed.post/qwertyuiop`
    pub uri: String,
    /// The post id. Eg: `qwertyuiop`
    pub post_id: &'a str,
    /// The author's repo, as a string. Eg: `did:plc:asdfghjkl`
    pub author: &'a str,
}

#[allow(dead_code)]
pub struct OnPostDeleteParams<'a> {
    pub commit: &'a Commit,
    /// The full uri. Eg: `at://did:plc:asdfghjkl/app.bsky.feed.post/qwertyuiop`
    pub uri: String,
    /// The post id. Eg: `qwertyuiop`
    pub post_id: &'a str,
    /// The author's repo, as a string. Eg: `did:plc:asdfghjkl`
    pub author: &'a str,
}

pub type OnPostCreate<DATA> = Arc<
    dyn for<'a> Fn(
            OnPostCreateParams<'a>,
            Arc<DATA>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
        + Send
        + Sync,
>;
pub type OnPostDelete<DATA> = Arc<
    dyn for<'a> Fn(
            OnPostDeleteParams<'a>,
            Arc<DATA>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
        + Send
        + Sync,
>;

pub struct Handler<DATA> {
    pub on_post_create: OnPostCreate<DATA>,
    pub on_post_delete: OnPostDelete<DATA>,
    pub data: Arc<DATA>,
}

impl<DATA: Send + Sync> CommitHandler for Handler<DATA> {
    async fn handle_commit(&self, commit: &Commit) -> Result<()> {
        for op in &commit.ops {
            // path is something like `app.bsky.feed.post/3lb3tt5kwha2w`
            // we only care about posts where the path starts with `app.bsky.feed.post`
            let Some((AtriumPost::NSID, post_id)) = op.path.split_once('/') else {
                continue;
            };

            // skip things that aren't creates
            if op.action == "create" {
                let uri = format!("at://{}/{}", commit.repo.as_str(), &op.path);

                let (items, _header) =
                    rs_car::car_read_all(&mut commit.blocks.as_slice(), true).await?;

                // get the referenced item out of the list
                let Some((_, item)) = items.iter().find(|(cid, _)| {
                    op.cid
                        .as_ref()
                        // TODO figure out how to do this equality without to_bytes
                        .map(|c| c.0.to_bytes() == cid.to_bytes())
                        .unwrap_or(false)
                }) else {
                    return Err(anyhow!(
                        "FAILED: could not find item with operation cid {:?} out of {} items",
                        op.cid,
                        items.len()
                    ));
                };

                let record = serde_ipld_dagcbor::from_reader::<Record, _>(&mut item.as_slice())?;

                let params = OnPostCreateParams {
                    post: &record,
                    commit,
                    uri,
                    post_id,
                    author: commit.repo.as_str(),
                };

                (self.on_post_create)(params, self.data.clone()).await;
            } else if op.action == "delete" {
                let uri = format!("at://{}/{}", commit.repo.as_str(), &op.path);

                let params = OnPostDeleteParams {
                    commit,
                    uri,
                    post_id,
                    author: commit.repo.as_str(),
                };

                (self.on_post_delete)(params, self.data.clone()).await;
            }
        }

        Ok(())
    }
}
