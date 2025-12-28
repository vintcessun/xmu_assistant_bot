use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Sex {
    Male,
    Female,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SenderPrivate {
    pub user_id: Option<i64>,
    pub nickname: Option<String>,
    pub sex: Option<Sex>,
    pub age: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Owner,
    Admin,
    Member,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SenderGroup {
    pub user_id: Option<i64>,
    pub nickname: Option<String>,
    pub card: Option<String>,
    pub sex: Option<Sex>,
    pub age: Option<i32>,
    pub area: Option<String>,
    pub level: Option<String>,
    pub role: Option<Role>,
    pub title: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum Sender {
    Private(SenderPrivate),
    Group(SenderGroup),
}
