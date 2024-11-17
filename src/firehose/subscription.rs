use anyhow::Result;
use atrium_api::com::atproto::sync::subscribe_repos::{Commit, NSID};
use futures::StreamExt;
use std::future::Future;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use super::stream::frames::Frame;

pub trait CommitHandler {
    fn handle_commit(&self, commit: &Commit) -> impl Future<Output = Result<()>> + Send;
}

pub struct RepoSubscription {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl RepoSubscription {
    pub async fn new(bgs: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, _) = connect_async(format!("wss://{bgs}/xrpc/{NSID}")).await?;
        Ok(RepoSubscription { stream })
    }

    pub async fn run(
        &mut self,
        handler: impl CommitHandler + Send + Sync + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let handler = Arc::new(handler);

        while let Some(result) = self.next().await {
            if let Ok(Frame::Message(Some(t), message)) = result {
                if t.as_str() == "#commit" {
                    let handler = handler.clone();
                    tokio::spawn(async move {
                        let Ok(commit) = serde_ipld_dagcbor::from_reader(message.body.as_slice())
                        else {
                            return;
                        };
                        if let Err(err) = handler.handle_commit(&commit).await {
                            eprintln!("FAILED: {err:?}");
                        }
                    });
                }
            }
        }
        Ok(())
    }

    async fn next(&mut self) -> Option<Result<Frame, <Frame as TryFrom<&[u8]>>::Error>> {
        if let Some(Ok(Message::Binary(data))) = self.stream.next().await {
            Some(Frame::try_from(data.as_slice()))
        } else {
            None
        }
    }
}
