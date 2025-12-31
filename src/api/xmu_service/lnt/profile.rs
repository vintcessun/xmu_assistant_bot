use crate::api::network::SessionClient;
use crate::api::xmu_service::lnt::LNT_URL;
use anyhow::Result;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use serde_json::value::RawValue;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct Department {
    pub id: i64,
    pub name: String,
    pub code: Option<Box<RawValue>>,
    pub cover: Option<Box<RawValue>>,
    pub created_at: Option<Box<RawValue>>,
    pub created_user: Option<Box<RawValue>>,
    pub is_show_on_homepage: Option<Box<RawValue>>,
    pub parent_id: Option<Box<RawValue>>,
    pub short_name: Option<Box<RawValue>>,
    pub sort: Option<Box<RawValue>>,
    pub stopped: Option<Box<RawValue>>,
    pub storage_assigned: Option<Box<RawValue>>,
    pub storage_used: Option<Box<RawValue>>,
    pub updated_at: Option<Box<RawValue>>,
    pub updated_user: Option<Box<RawValue>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProfileResponse {
    pub id: i64,
    pub name: String,
    pub user_no: String,
    pub active: Option<Box<RawValue>>,
    pub ai_activation: Option<Box<RawValue>>,
    pub audit: Option<Box<RawValue>>,
    pub avatar_big_url: Option<Box<RawValue>>,
    pub avatar_small_url: Option<Box<RawValue>>,
    pub comment: Option<Box<RawValue>>,
    pub created_at: Option<Box<RawValue>>,
    pub created_by: Option<Box<RawValue>>,
    pub department: Department,
    pub education: Option<Box<RawValue>>,
    pub email: Option<Box<RawValue>>,
    pub end_at: Option<Box<RawValue>>,
    pub grade: Option<Box<RawValue>>,
    pub has_ai_ability: Option<Box<RawValue>>,
    pub imported_from: Option<Box<RawValue>>,
    pub is_imported_data: Option<Box<RawValue>>,
    pub klass: Option<Box<RawValue>>,
    pub mobile_phone: Option<Box<RawValue>>,
    pub nickname: Option<Box<RawValue>>,
    pub org: Option<Box<RawValue>>,
    pub program: Option<Box<RawValue>>,
    pub program_id: Option<Box<RawValue>>,
    pub remarks: Option<Box<RawValue>>,
    pub require_verification: Option<Box<RawValue>>,
    pub role: Option<Box<RawValue>>,
    pub user_addresses: Option<Box<RawValue>>,
    pub user_attributes: Option<Box<RawValue>>,
    pub user_auth_externals: Option<Box<RawValue>>,
    pub user_personas: Option<Box<RawValue>>,
    pub webex_auth: Option<Box<RawValue>>,
}

static PROFILE: Lazy<ProfileStruct> = Lazy::new(ProfileStruct::new);

pub struct ProfileStruct {
    pub profile_data: DashMap<String, Arc<ProfileResponse>>,
}

impl ProfileStruct {
    pub fn new() -> Self {
        ProfileStruct {
            profile_data: DashMap::new(),
        }
    }

    pub async fn get_profile(&self, session: &str) -> Result<Arc<ProfileResponse>> {
        if let Some(entry) = self.profile_data.get(session) {
            return Ok((*entry.value()).clone());
        }

        let client = SessionClient::new();
        client.set_cookie("session", session, LNT_URL.clone());

        let res = client.get("https://lnt.xmu.edu.cn/api/profile").await?;
        let user_info = res.json::<ProfileResponse>().await?;
        let user_info = Arc::new(user_info);

        self.profile_data
            .insert(session.to_string(), user_info.clone());
        Ok(user_info)
    }
}

pub struct Profile;

impl Profile {
    pub async fn get_profile(session: &str) -> Result<Arc<ProfileResponse>> {
        PROFILE.get_profile(session).await
    }
}
