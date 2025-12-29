use helper::api;
use serde::{Deserialize, Serialize};

use crate::abi::{
    echo::Echo,
    message::{MessageSend, api::data, message_body},
};

pub trait Params: Send + Sync + 'static + Serialize {
    type Response: data::ApiResponseTrait + for<'de> Deserialize<'de>;

    const ACTION: &'static str;
}

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
    pub fn new(group_id: i64, messages: Vec<MessageSend>) -> Self {
        Self {
            group_id,
            messages: MessageSend::Array(
                messages
                    .into_iter()
                    .map(|m| {
                        message_body::SegmentSend::Node(Box::new(
                            message_body::node::DataSend::Content(message_body::node::DataSend2 {
                                user_id: "114514".to_string(),
                                nickname: "聊天转发".to_string(),
                                content: Box::new(m),
                            }),
                        ))
                    })
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[api("/group_poke", data::PokeResponse)]
pub struct GroupPoke {
    group_id: i64,
    user_id: i64,
}

impl GroupPoke {
    pub const fn new(group_id: i64, user_id: i64) -> Self {
        Self { group_id, user_id }
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
    pub fn new(user_id: i64, messages: Vec<MessageSend>) -> Self {
        Self {
            user_id,
            messages: MessageSend::Array(
                messages
                    .into_iter()
                    .map(|m| {
                        message_body::SegmentSend::Node(Box::new(
                            message_body::node::DataSend::Content(message_body::node::DataSend2 {
                                user_id: "".to_string(),
                                nickname: "".to_string(),
                                content: Box::new(m),
                            }),
                        ))
                    })
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[api("/friend_poke", data::PokeResponse)]
pub struct FriendPoke {
    user_id: i64,
    target_id: Option<i64>,
}

impl FriendPoke {
    pub const fn new(user_id: i64) -> Self {
        Self {
            user_id,
            target_id: None,
        }
    }
}
