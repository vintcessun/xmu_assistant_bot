use crate::{abi::message::MessageSend, api::storage::ColdTable};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static MESSAGE_FAST_DB: LazyLock<ColdTable<MessageAbstract, MessageSend>> =
    LazyLock::new(|| ColdTable::new("message_fast_abstract_reply"));

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct MessageAbstract {
    pub qq: i64,
    pub msg_text: String,
}

pub async fn get_repeat_reply(key: MessageAbstract) -> Option<MessageSend> {
    MESSAGE_FAST_DB.get(key).await.unwrap_or_default()
}

pub async fn insert_repeat_reply(key: MessageAbstract, message: MessageSend) {
    let _ = MESSAGE_FAST_DB.insert(key, message).await;
}
