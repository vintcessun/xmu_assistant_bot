use super::LoginData;
use super::main::DATA;
use crate::abi::message::MessageSend;
use crate::api::xmu_service::IDS_URL;
use crate::api::xmu_service::jw::Zzy;
use crate::api::xmu_service::lnt::{LNT_URL, LoginRequest, Profile};
use crate::{abi::logic_import::*, api::network::SessionClient};
use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;
use std::time;
use tracing::{debug, trace};
use url::Url;
use url_macro::url;

static LOGIN_URL: Lazy<Url> =
    Lazy::new(|| url!("https://jw.xmu.edu.cn/login?service=https://jw.xmu.edu.cn/new/index.html"));

lazy_static! {
    static ref REGEX_EXECUTION: Arc<Regex> = Arc::new(
        Regex::new("<input[^>]*?name=\"execution\"[^>]*?value=\"([^\"]*)\"[^>]*?>").unwrap()
    );
}

async fn get_qrcode(session: &SessionClient) -> Result<LoginRequest> {
    let login_page = session.get(LOGIN_URL.clone()).await?;
    let base_url = login_page.url().to_string();
    let login_page_text = login_page.text().await?;
    if login_page_text.contains("IP冻结提示") {
        return Err(anyhow!("登录服务被冻结，请联系管理员解决。".to_string(),));
    }
    if !login_page_text.contains("qrLoginForm") {
        return Err(anyhow!("登录错误，可能是登录页面结构发生了变化。"));
    }

    let login_form_data = &login_page_text[login_page_text.find("qrLoginForm").unwrap()..];

    //找到第一个符合要求的
    let execution = REGEX_EXECUTION
        .captures(login_form_data)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .ok_or_else(|| anyhow!("无法找到execution字段"))?
        .to_string();

    let resp = session
        .get(
            format!(
                "https://ids.xmu.edu.cn/authserver/qrCode/getToken?ts={}",
                time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)?
                    .as_millis()
            )
            .as_str(),
        )
        .await?;

    let qrcode_id = resp.text().await?.trim().to_string();

    Ok(LoginRequest::qrcode(base_url, qrcode_id, execution))
}

async fn wait_qrcode(session: &SessionClient, qrcode_id: &str) -> Result<()> {
    loop {
        let status = session
            .get(format!(
                "https://ids.xmu.edu.cn/authserver/qrCode/getStatus.htl?ts={}&uuid={}",
                time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)?
                    .as_millis(),
                qrcode_id
            ))
            .await?
            .text()
            .await?;
        match status.as_str() {
            "0" => {
                trace!("二维码未扫码，继续等待");
            }
            "1" => {
                debug!("请求成功");
                break;
            }
            "2" => {
                trace!("二维码已扫码，等待确认");
            }
            "3" => {
                return Err(anyhow!("二维码已失效，请重新登录。"));
            }
            _ => {
                return Err(anyhow!("未知的二维码状态码。"));
            }
        }
        tokio::time::sleep(time::Duration::from_secs(1)).await;
    }
    Ok(())
}

async fn request_qrcode(session: &SessionClient, data: LoginRequest) -> Result<LoginData> {
    session
        .post(&data.url, &data.body)
        .await?
        .error_for_status_ref()?;

    let castgc = session
        .get_cookie("CASTGC", &IDS_URL)
        .ok_or(anyhow!("登录失败，未获取到CASTGC Cookie"))?;

    let _ = session.get(LNT_URL.clone()).await?.error_for_status()?;

    let lnt = session
        .get_cookie("session", &LNT_URL)
        .ok_or(anyhow!("登录失败，未获取到session"))?;

    Ok(LoginData {
        castgc: castgc.to_string(),
        lnt: lnt.to_string(),
    })
}

pub async fn process_login<T: BotClient + BotHandler + fmt::Debug>(
    ctx: &mut Context<T, Message>,
    id: i64,
) -> Result<()> {
    let session = SessionClient::new();

    let data = get_qrcode(&session).await?;

    let qrcode_id = data
        .body
        .qrcode_id
        .clone()
        .ok_or(anyhow!("二维码生成失败"))?;

    let qrcode_url = format!(
        "https://ids.xmu.edu.cn/authserver/qrCode/getCode?uuid={}",
        qrcode_id
    );

    ctx.send_message(
        MessageSend::new_message()
            .text("请使用企业微信扫码登录：")
            .image(qrcode_url)
            .build(),
    )
    .await?;

    wait_qrcode(&session, &qrcode_id).await?;

    let login_data = Arc::new(request_qrcode(&session, data).await?);

    let login_data_insert = login_data.clone();

    DATA.insert(id, login_data_insert)?;

    ctx.send_message_async(message::from_str("登录成功！"));

    let profile = Profile::get_profile(&login_data.lnt).await?;

    let data = Zzy::get(&login_data.castgc, &profile.user_no).await?;

    let zzy_profile = data.get_profile()?;

    ctx.send_message_async(message::from_str(format!(
        "信息:{} 转入学院:{:?}",
        zzy_profile.entry_year, zzy_profile.trans_dept
    )));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_login_process() -> Result<()> {
        let session = SessionClient::new();

        let data = get_qrcode(&session).await?;

        println!("数据：{}", serde_json::to_string(&data)?);

        let qrcode_id = data
            .body
            .qrcode_id
            .clone()
            .ok_or(anyhow!("二维码生成失败"))?;

        let qrcode_url = format!(
            "https://ids.xmu.edu.cn/authserver/qrCode/getCode?uuid={}",
            qrcode_id
        );

        println!("请使用企业微信扫码登录：{}", qrcode_url);

        wait_qrcode(&session, &qrcode_id).await?;

        let login_data = Arc::new(request_qrcode(&session, data).await?);

        println!("登录成功！");

        let profile = Profile::get_profile(&login_data.lnt).await?;

        println!("用户信息：{:?}", profile);

        let data = Zzy::get(&login_data.castgc, &profile.user_no).await?;

        let zzy_profile = data.get_profile()?;

        println!(
            "信息:{} 转入学院:{:?}",
            zzy_profile.entry_year, zzy_profile.trans_dept
        );

        Ok(())
    }
}
