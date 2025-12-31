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
    let (adapter, subscribe) = network::NapcatAdapter::new();
    let mut client = websocket::BotWebsocketClient::new(config, adapter);
    client.connect().await?;
    let router = NapcatRouter::new(subscribe, client);
    Ok(router)
}

pub mod logic_import {
    pub use crate::abi::message;
    pub use crate::abi::{
        Context, Handler,
        message::{
            MessageType, Target, event::Type, event_message::Message, event_notice::Notice,
            event_request::Request,
        },
        network::BotClient,
        websocket::BotHandler,
    };
    pub use crate::config;
    pub use helper::handler;
    pub use helper::register_handlers;
    pub use std::fmt;
}
