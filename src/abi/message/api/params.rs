use helper::api;
use serde::{Deserialize, Serialize};

use crate::abi::{
    echo::Echo,
    message::{MessageSend, api::data},
};

pub trait Params: Send + Sync + 'static + Serialize {
    type Response: data::ApiResponseTrait + for<'de> Deserialize<'de>;

    fn get_action(&self) -> &'static str;
}

#[derive(Serialize, Debug)]
pub struct ApiSend<T: Params> {
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
    pub fn new(group_id: i64, message: MessageSend) -> Self {
        Self { group_id, message }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForwardNode {
    pub uin: Option<i64>,
    pub name: Option<String>,
    pub content: Vec<MessageSend>,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ForwardMessage {
    Node { data: ForwardNode },
}

#[derive(Serialize, Debug)]
#[api("/send_group_forward_msg", data::ForwardMsgResponse)]
pub struct SendGroupForwardMessageParams {
    pub group_id: i64,
    pub messages: Vec<ForwardMessage>,
}

impl SendGroupForwardMessageParams {
    pub fn new(group_id: i64, messages: Vec<MessageSend>) -> Self {
        Self {
            group_id,
            messages: vec![ForwardMessage::Node {
                data: ForwardNode {
                    uin: None,
                    name: None,
                    content: messages,
                },
            }],
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
    pub fn new(group_id: i64, user_id: i64) -> Self {
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
    pub fn new(user_id: i64, message: MessageSend) -> Self {
        Self { user_id, message }
    }
}

#[derive(Serialize, Debug)]
#[api("/send_private_forward_msg", data::ForwardMsgResponse)]
pub struct SendPrivateForwardMessageParams {
    pub user_id: i64,
    pub messages: Vec<ForwardMessage>,
}

impl SendPrivateForwardMessageParams {
    pub fn new(user_id: i64, messages: Vec<MessageSend>) -> Self {
        Self {
            user_id,
            messages: vec![ForwardMessage::Node {
                data: ForwardNode {
                    uin: None,
                    name: None,
                    content: messages,
                },
            }],
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
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            target_id: None,
        }
    }
}
