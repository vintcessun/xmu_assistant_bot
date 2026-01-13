use genai::chat::ChatMessage;

use crate::api::storage::ColdTable;
use std::sync::LazyLock;

static MESSAGE_DB: LazyLock<ColdTable<String, ChatMessage>> =
    LazyLock::new(|| ColdTable::new("llm_chat_message_storage"));

static NOTICE_DB: LazyLock<ColdTable<i64, ChatMessage>> =
    LazyLock::new(|| ColdTable::new("llm_chat_notice_storage"));

pub struct MessageStorage;

impl MessageStorage {
    pub async fn get(key: String) -> Option<ChatMessage> {
        MESSAGE_DB.get(key).await.unwrap_or_default()
    }

    pub async fn save(key: String, message: Vec<ChatMessage>) {
        let mut msg_contents = vec![];
        for msg in message {
            msg_contents.extend(msg.content);
        }
        let _ = MESSAGE_DB
            .insert(key, ChatMessage::user(msg_contents))
            .await;
    }
}

pub struct NoticeStorage;

impl NoticeStorage {
    pub async fn get(key: i64) -> Option<ChatMessage> {
        NOTICE_DB.get(key).await.unwrap_or_default()
    }

    pub async fn save(key: i64, message: ChatMessage) {
        let _ = NOTICE_DB.insert(key, message).await;
    }
}
