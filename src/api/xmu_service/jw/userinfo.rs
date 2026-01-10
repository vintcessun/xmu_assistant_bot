use super::JwAPI;
use anyhow::Result;
use helper::jw_api;
//use serde::de::IgnoredAny;
use serde::Deserialize;

#[jw_api(
    url = "https://jw.xmu.edu.cn/jwapp/sys/jwai/api/user/getCurrentUser.do",
    app = "https://jw.xmu.edu.cn/new/index.html",
    auto_row = false,
    call_type = "get"
)]
pub struct UserInfo {
    pub user_id: String, // 用户ID
    pub user_name: String, // 用户名称
                         //pub rzlbdm: String,    // 认证类别代码
}

impl UserInfo {
    pub async fn get_userinfo(castgc: &str) -> Result<UserInfoDataApi> {
        let userinfo = Self::call(castgc).await?;
        Ok(userinfo.datas.getCurrentUser)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test() -> Result<()> {
        let castgc = "TGT-2435869-O8Wwbqik8mV2AiaFWm2RKkKG8nq1zARLvjuN2XWuYtBMaXNrSUaZDng4bJZj-3FfQrsnull_main";
        let user_info = UserInfo::call(castgc).await?;
        println!("UserInfo API Response: {:?}", user_info);
        Ok(())
    }
}
