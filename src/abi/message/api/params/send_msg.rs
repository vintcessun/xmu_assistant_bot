use core::panic;
use std::{
    any::{Any, TypeId},
    fmt::{self},
};

use super::Params;
use crate::{
    abi::{
        Context,
        echo::Echo,
        message::{
            self, MessageSend, MessageType, Target,
            api::data,
            event_body,
            message_body::{self, SegmentSend},
        },
        network::BotClient,
        websocket::BotHandler,
    },
    box_new,
};
use helper::api;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

#[derive(Serialize, Debug)]
pub struct ApiSend<T: Params + Serialize> {
    pub action: &'static str,
    pub params: T,
    pub echo: Echo,
}

#[derive(Serialize, Deserialize, Debug)]
#[api("/send_group_msg", data::SendMsgResponse)]
pub struct SendGroupMessageParams {
    pub group_id: i64,
    pub message: MessageSend,
}

impl SendGroupMessageParams {
    pub const fn new(group_id: i64, message: MessageSend) -> Self {
        Self { group_id, message }
    }
}

#[derive(Serialize, Debug)]
#[api("/send_group_forward_msg", data::ForwardMsgResponse)]
pub struct SendGroupForwardMessageParams {
    pub group_id: i64,
    pub messages: MessageSend,
}

impl SendGroupForwardMessageParams {
    pub fn new<T, M>(ctx: Context<T, M>) -> Self
    where
        T: BotClient + BotHandler + fmt::Debug + Send + Sync + 'static,
        M: MessageType + fmt::Debug + Send + Sync + 'static,
    {
        let group_id = match ctx.get_target() {
            Target::Group(group_id) => group_id,
            _ => panic!("SendGroupForwardMessageParams 只能用于群聊消息"),
        };
        let message = get_msg(ctx);

        Self {
            group_id,
            messages: MessageSend::Array(message),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[api("/send_private_msg", data::SendMsgResponse)]
pub struct SendPrivateMessageParams {
    pub user_id: i64,
    pub message: MessageSend,
}

impl SendPrivateMessageParams {
    pub const fn new(user_id: i64, message: MessageSend) -> Self {
        Self { user_id, message }
    }
}

#[derive(Serialize, Debug)]
#[api("/send_private_forward_msg", data::ForwardMsgResponse)]
pub struct SendPrivateForwardMessageParams {
    pub user_id: i64,
    pub messages: MessageSend,
}

impl SendPrivateForwardMessageParams {
    pub fn new<T, M>(ctx: Context<T, M>) -> Self
    where
        T: BotClient + BotHandler + fmt::Debug + Send + Sync + 'static,
        M: MessageType + fmt::Debug + Send + Sync + 'static,
    {
        let user_id = match ctx.get_target() {
            Target::Private(user_id) => user_id,
            _ => panic!("SendPrivateForwardMessageParams 只能用于私聊消息"),
        };
        let msg = get_msg(ctx);
        Self {
            user_id,
            messages: MessageSend::Array(msg),
        }
    }
}

fn get_msg<T, M>(mut ctx: Context<T, M>) -> Vec<SegmentSend>
where
    T: BotClient + BotHandler + fmt::Debug + Send + Sync + 'static,
    M: MessageType + fmt::Debug + Send + Sync + 'static,
{
    let mut message = Vec::with_capacity(ctx.message_list.len() + 3);
    if ctx.is_echo {
        if TypeId::of::<M>() != TypeId::of::<event_body::message::Message>() {
            panic!("只能对接收的消息使用 echo_cmd");
        } else {
            let sender = ctx.sender.clone();
            let ctx = (&ctx as &dyn Any)
                .downcast_ref::<Context<T, event_body::message::Message>>()
                .unwrap();
            let msg = ctx.get_message();
            let msg_content = match &*msg {
                event_body::message::Message::Private(p) => &p.message,
                event_body::message::Message::Group(g) => &g.message,
            };
            let msg_add = message::receive2send_add_prefix(
                msg_content,
                match ctx.get_target() {
                    Target::Group(group_id) => format!(
                        "来自群({group_id})的{}({} {})命令: ",
                        ctx.sender
                            .card
                            .as_ref()
                            .unwrap_or(&String::from("未知群昵称")),
                        &ctx.sender
                            .nickname
                            .as_ref()
                            .unwrap_or(&String::from("未知昵称")),
                        &ctx.sender.user_id.unwrap_or(0),
                    ),
                    Target::Private(user_id) => {
                        format!(
                            "用户{user_id}({})的命令: ",
                            &ctx.sender
                                .nickname
                                .as_ref()
                                .unwrap_or(&String::from("未知昵称"))
                        )
                    }
                },
            );
            message.push(message_body::SegmentSend::Node(box_new!(
                message_body::node::DataSend,
                message_body::node::DataSend::Content(message_body::node::DataSend2 {
                    user_id: format!("{}", sender.user_id.unwrap_or(114514)),
                    nickname: sender
                        .nickname
                        .as_ref()
                        .unwrap_or(&"用户指令".to_string())
                        .to_owned(),
                    content: box_new!(MessageSend, msg_add),
                })
            )))
        }
    };

    let messages = std::mem::take(&mut ctx.message_list);
    debug!("发送转发消息共{}条", messages.len());
    trace!(?messages);
    for msg in messages {
        message.push(message_body::SegmentSend::Node(box_new!(
            message_body::node::DataSend,
            message_body::node::DataSend::Content(message_body::node::DataSend2 {
                user_id: "1363408373".to_string(),
                nickname: "指令回复".to_string(),
                content: box_new!(MessageSend, msg.clone()),
            })
        )))
    }
    message
}
