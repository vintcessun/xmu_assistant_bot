mod echo;
pub mod message;
pub mod network;
pub mod router;
pub mod websocket;

use anyhow::Result;
pub use router::context::Context;
pub use router::handler::Handler;
use router::handler::NapcatRouter;

use crate::{
    abi::{network::NapcatAdapter, router::handler::Router},
    config::ServerConfig,
};

pub async fn run(config: ServerConfig) -> Result<NapcatRouter<NapcatAdapter>> {
    let (adapter, subscribe) = network::NapcatAdapter::new(1024);
    let mut client = websocket::BotWebsocketClient::new(config, adapter, 32, 32);
    client.connect().await?;
    let router = NapcatRouter::new(subscribe, client);
    Ok(router)
}

pub mod logic_import {
    pub use crate::abi::{
        Context, Handler,
        message::{MessageType, Target, event::Type, event_message::Message},
        network::BotClient,
        websocket::BotHandler,
    };

    pub use crate::abi::message;
}
