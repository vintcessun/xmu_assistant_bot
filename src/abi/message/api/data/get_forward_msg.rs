use crate::abi::message::{
    MessageReceive,
    api::{ApiResponse, Data},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetForwardMsgData {
    #[serde(alias = "message")]
    pub messages: MessageReceive,
}

impl Data for GetForwardMsgData {}

pub type GetForwardMsgResponse = ApiResponse<GetForwardMsgData>;
