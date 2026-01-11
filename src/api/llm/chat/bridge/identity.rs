use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub qq: i64,
    pub group: i64,
    pub nickname: String,
    pub username: String,
}
