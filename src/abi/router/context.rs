use crate::abi::echo::Echo;
use crate::abi::message::MessageSend;
use crate::abi::message::Sender;
use crate::abi::message::api;
use crate::abi::message::{MessageType, Target};
use crate::abi::network::BotClient;
use crate::abi::websocket::BotHandler;
use anyhow::Result;
use std::fmt;
use std::sync::Arc;
use tracing::{error, info, trace};

#[derive(Debug)]
pub struct Context<
    T: BotClient + BotHandler + fmt::Debug + Send + Sync + 'static,
    M: MessageType + fmt::Debug + Send + Sync + 'static,
> {
    pub client: Arc<T>,
    pub message: Arc<M>,
    pub sender: Arc<Sender>,
    pub message_list: Vec<MessageSend>,
    pub message_text: Arc<str>,
    pub target: Target,
    pub is_echo: bool,
}

impl<
    T: BotClient + BotHandler + fmt::Debug + Send + Sync + 'static,
    M: MessageType + fmt::Debug + Send + Sync + 'static,
> Clone for Context<T, M>
{
    fn clone(&self) -> Self {
        Context {
            client: self.client.clone(),
            message: self.message.clone(),
            sender: self.sender.clone(),
            message_list: self.message_list.clone(),
            message_text: self.message_text.clone(),
            target: self.target,
            is_echo: self.is_echo,
        }
    }
}

impl<
    T: BotClient + BotHandler + fmt::Debug + Send + Sync + 'static,
    M: MessageType + fmt::Debug + Send + Sync + 'static,
> Context<T, M>
{
    pub fn new(client: Arc<T>, message: Arc<M>) -> Self {
        let target = message.get_target();
        let message_text = message.get_text();
        let sender = message.get_sender();
        let message_list = Vec::new();
        Context {
            client,
            message,
            sender: Arc::from(sender),
            target,
            message_list,
            message_text: Arc::from(message_text),
            is_echo: false,
        }
    }

    pub fn set_echo(&mut self) {
        self.is_echo = true;
    }

    pub async fn send_message(&self, message: MessageSend) -> Result<()> {
        match self.target {
            Target::Group(group_id) => {
                let params = api::SendGroupMessageParams::new(group_id, message);
                let call = self.client.call_api(params, Echo::new()).await?;
                let res = call.wait_echo().await?;
                trace!(?res);
                match res.status {
                    api::Status::Ok => Ok(()),
                    api::Status::Failed => Err(anyhow::anyhow!(
                        "发送群消息失败: {:?}",
                        res.message.unwrap_or("未知错误".to_string())
                    )),
                    api::Status::Async => Err(anyhow::anyhow!("发送群消息异步处理中")),
                }
            }
            Target::Private(user_id) => {
                let params = api::SendPrivateMessageParams::new(user_id, message);
                let call = self.client.call_api(params, Echo::new()).await?;
                let res = call.wait_echo().await?;
                trace!(?res);
                match res.status {
                    api::Status::Ok => Ok(()),
                    api::Status::Failed => Err(anyhow::anyhow!(
                        "发送私聊消息失败: {:?}",
                        res.message.unwrap_or("未知错误".to_string())
                    )),
                    api::Status::Async => Err(anyhow::anyhow!("发送私聊消息异步处理中")),
                }
            }
        }
    }

    pub fn send_message_async(&mut self, message: MessageSend) {
        self.message_list.push(message);
    }

    pub fn get_message(&self) -> Arc<M> {
        self.message.clone()
    }

    pub fn get_message_text(&self) -> &str {
        &self.message_text
    }

    pub fn get_target(&self) -> Target {
        self.target
    }

    pub async fn set_title(&self, title: String) -> Result<()> {
        let params = api::SpecialTitle::new(
            match self.target {
                Target::Group(group_id) => group_id,
                Target::Private(_) => {
                    return Err(anyhow::anyhow!(
                        "只能在群聊中设置特殊头衔，当前目标不是群聊"
                    ));
                }
            },
            self.sender.user_id.unwrap_or(0),
            title,
        );
        let call = self.client.call_api(params, Echo::new()).await?;
        let res = call.wait_echo().await?;
        trace!(?res);
        match res.status {
            api::Status::Ok => Ok(()),
            api::Status::Failed => Err(anyhow::anyhow!(
                "设置特殊头衔失败: {:?}",
                res.message.unwrap_or("未知错误".to_string())
            )),
            api::Status::Async => Err(anyhow::anyhow!("设置特殊头衔异步处理中")),
        }
    }
}

impl<
    T: BotClient + BotHandler + fmt::Debug + Send + Sync + 'static,
    M: MessageType + fmt::Debug + Send + Sync + 'static,
> Drop for Context<T, M>
{
    fn drop(&mut self) {
        if !self.message_list.is_empty() {
            let client = self.client.clone();
            let target = self.target;
            let ctx = Context {
                client: self.client.clone(),
                message: self.message.clone(),
                sender: self.sender.clone(),
                message_list: std::mem::take(&mut self.message_list),
                message_text: self.message_text.clone(),
                target: self.target,
                is_echo: self.is_echo,
            };
            tokio::spawn(async move {
                match target {
                    Target::Group(_) => {
                        let params = api::SendGroupForwardMessageParams::new(ctx);
                        let call = client.call_api(params, Echo::new()).await;
                        if let Ok(call) = call {
                            let res = call.wait_echo().await;
                            trace!(?res);
                            if let Ok(res) = res {
                                match res.status {
                                    api::Status::Ok => {}
                                    api::Status::Failed => {
                                        error!(
                                            "发送群转发消息失败: {:?}",
                                            res.message.unwrap_or("未知错误".to_string())
                                        );
                                    }
                                    api::Status::Async => {
                                        info!("发送群转发消息异步处理中");
                                    }
                                }
                            }
                        }
                    }
                    Target::Private(_) => {
                        let params = api::SendPrivateForwardMessageParams::new(ctx);
                        let call = client.call_api(params, Echo::new()).await;
                        if let Ok(call) = call {
                            let res = call.wait_echo().await;
                            trace!(?res);
                            if let Ok(res) = res {
                                match res.status {
                                    api::Status::Ok => {}
                                    api::Status::Failed => {
                                        error!(
                                            "发送私聊转发消息失败: {:?}",
                                            res.message.unwrap_or("未知错误".to_string())
                                        );
                                    }
                                    api::Status::Async => {
                                        info!("发送私聊转发消息异步处理中");
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    }
}
