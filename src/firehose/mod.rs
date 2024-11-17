use anyhow::{anyhow, Result};
use atrium_api::{
    app::bsky::feed::{
        post::{Record, RecordData as PostRecordData},
        Post,
    },
    com::atproto::sync::subscribe_repos::{Commit, NSID},
    types::{Collection, Object},
};
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use stream::frames::Frame;
use subscription::{CommitHandler, Subscription};

mod stream;
mod subscription;

struct RepoSubscription {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl RepoSubscription {
    async fn new(bgs: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, _) = connect_async(format!("wss://{bgs}/xrpc/{NSID}")).await?;
        Ok(RepoSubscription { stream })
    }
    async fn run(&mut self, handler: impl CommitHandler) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(result) = self.next().await {
            if let Ok(Frame::Message(Some(t), message)) = result {
                if t.as_str() == "#commit" {
                    let commit = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;
                    if let Err(err) = handler.handle_commit(&commit).await {
                        eprintln!("FAILED: {err:?}");
                    }
                }
            }
        }
        Ok(())
    }
}

impl Subscription for RepoSubscription {
    async fn next(&mut self) -> Option<Result<Frame, <Frame as TryFrom<&[u8]>>::Error>> {
        if let Some(Ok(Message::Binary(data))) = self.stream.next().await {
            Some(Frame::try_from(data.as_slice()))
        } else {
            None
        }
    }
}

struct Firehose {
    on_post: Box<dyn Fn(&Object<PostRecordData>, &Commit)>,
}

impl CommitHandler for Firehose {
    async fn handle_commit(&self, commit: &Commit) -> Result<()> {
        for op in &commit.ops {
            // path is something like `app.bsky.feed.post/3lb3tt5kwha2w`
            // we only care about posts where the path starts with `app.bsky.feed.post`
            let Some(Post::NSID) = op.path.split('/').next() else {
                continue;
            };

            // skip things that aren't creates
            if op.action != "create" {
                continue;
            }

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

            (self.on_post)(&record, commit);
        }
        Ok(())
    }
}

pub async fn listen(
    on_post: impl Fn(&Object<PostRecordData>, &Commit) + 'static,
) -> Result<(), Box<dyn std::error::Error>> {
    RepoSubscription::new("bsky.network")
        .await?
        .run(Firehose {
            on_post: Box::new(on_post),
        })
        .await
}
