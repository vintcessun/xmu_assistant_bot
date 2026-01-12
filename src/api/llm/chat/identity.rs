use std::sync::LazyLock;

use ahash::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::api::storage::ColdTable;

static IDENTITY_DB: LazyLock<ColdTable<i64, Identity>> =
    LazyLock::new(|| ColdTable::new("llm_chat_identity"));

static UPDATE: LazyLock<IdentityUpdate> = LazyLock::new(IdentityUpdate::new);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name {
    pub now: String,
    pub used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: i64,
    pub now_nickname: Name,
    pub group_nickname: HashMap<i64, Name>, //group_id -> nickname
}

pub async fn search_identity(qq: i64) -> Option<Identity> {
    IDENTITY_DB.get(qq).await.unwrap_or_default()
}

pub fn update_identity(qq: i64, group_id: i64) {
    IdentityUpdate::update(qq, group_id);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityUpdateSend {
    pub qq: i64,
    pub group_id: i64,
}

pub struct IdentityUpdate {
    pub update_channel: mpsc::UnboundedSender<IdentityUpdateSend>,
}

impl IdentityUpdate {
    pub fn update(qq: i64, group_id: i64) {
        let send = IdentityUpdateSend { qq, group_id };
        let _ = UPDATE.update_channel.send(send);
    }

    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<IdentityUpdateSend>();
        tokio::spawn(async move {
            while let Some(update) = rx.recv().await {
                if let Some(identity) = IDENTITY_DB.get(update.qq).await.unwrap_or_default() {
                    //TODO:更新身份API接入
                    todo!("更新身份信息: {:?}", identity);
                }
            }
        });
        IdentityUpdate { update_channel: tx }
    }
}
