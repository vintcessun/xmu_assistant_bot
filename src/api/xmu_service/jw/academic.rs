use super::IDS_URL;
use super::JwAPI;
use crate::api::network::SessionClient;
use anyhow::Result;
use helper::jw_api;
use serde::{Deserialize, Serialize};

#[jw_api(
    url = "https://jw.xmu.edu.cn/jwapp/sys/cjcx/modules/cjcx/cxycjdxnxq.do",
    app = "https://jw.xmu.edu.cn/appShow?appId=4768574631264620"
)]
pub struct Academic {}
