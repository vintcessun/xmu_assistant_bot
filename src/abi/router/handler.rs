use crate::abi::{
    message::{Event, MessageType, event_message::Message, event_meta::MetaEvent},
    network::BotClient,
    router::context::Context,
    websocket::{BotHandler, BotWebsocketClient},
};
use anyhow::Result;
use async_trait::async_trait;
use std::{fmt, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, error, trace};

#[async_trait]
pub trait Handler<T, M>: Send + Sync
where
    T: BotClient + BotHandler + fmt::Debug,
    M: MessageType + fmt::Debug,
{
    async fn handle(&self, ctx: Context<T, M>) -> Result<()>;
}

#[async_trait]
pub trait Router<T>
where
    T: BotClient + BotHandler + fmt::Debug,
{
    fn new(subscribe: mpsc::Receiver<Event>, client: BotWebsocketClient<T>) -> Self;
    fn get_client(&self) -> Arc<T>;
    async fn run(&mut self) -> ();
    fn spawn_context<M: MessageType + fmt::Debug + Send + Sync + 'static>(&self, msg: Arc<M>) {
        let client_arc = self.get_client();
        let context = Context::new(client_arc, msg);

        use crate::logic::EchoHandler;

        tokio::spawn(async move {
            let ctx = context.clone();
            EchoHandler.handle(ctx).await.unwrap_or_else(|e| {
                error!("处理消息时出错: {:?}", e);
            });
        });
    }
}

pub struct NapcatRouter<T: BotHandler> {
    subscribe: mpsc::Receiver<Event>,
    client: BotWebsocketClient<T>,
}

#[async_trait]
impl<T: BotHandler + BotClient + fmt::Debug> Router<T> for NapcatRouter<T> {
    fn new(subscribe: mpsc::Receiver<Event>, client: BotWebsocketClient<T>) -> Self {
        NapcatRouter { subscribe, client }
    }

    fn get_client(&self) -> Arc<T> {
        self.client.handler.clone()
    }

    async fn run(&mut self) {
        while let Some(event) = self.subscribe.recv().await {
            match event {
                Event::Message(msg) => {
                    debug!("处理消息事件: {:?}", msg);
                    self.spawn_context(Arc::<Message>::from(msg));
                }
                Event::Notice(notice) => {
                    debug!("处理通知事件: {:?}", notice);
                    self.spawn_context(Arc::from(notice));
                }
                Event::Request(req) => {
                    debug!("处理请求事件: {:?}", req);
                    self.spawn_context(Arc::from(req));
                }
                Event::MetaEvent(meta) => {
                    debug!("处理元事件: {:?}", meta);

                    match meta {
                        MetaEvent::Heartbeat(hb) => {
                            trace!("收到心跳事件: {:?}", hb);
                        }
                        MetaEvent::Lifecycle(lc) => {
                            trace!("收到生命周期事件: {:?}", lc);
                        }
                    }
                }
            }
        }
    }
}
