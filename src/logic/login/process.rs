use super::main::DATA;
use crate::abi::message::MessageSend;
use crate::api::xmu_service::jw::{Zzy, ZzyProfile};
use crate::api::xmu_service::lnt::Profile;
use crate::api::xmu_service::login::{LoginRequest, get_qrcode_id, request_qrcode, wait_qrcode};
use crate::{abi::logic_import::*, api::network::SessionClient};
use anyhow::Result;
use std::sync::Arc;

#[inline(never)]
pub async fn update_db_and_login_base(
    session: &SessionClient,
    data: LoginRequest,
    id: i64,
) -> Result<ZzyProfile> {
    let login_data = Arc::new(request_qrcode(session, data).await?);

    let login_data_insert = login_data.clone();

    DATA.insert(id, login_data_insert)?;

    let profile = Profile::get(&login_data.lnt).await?;

    let data = Zzy::get(&login_data.castgc, &profile.user_no).await?;

    let zzy_profile = data.get_profile()?;

    Ok(zzy_profile)
}

#[inline(never)]
pub async fn send_msg_and_wait<T: BotClient + BotHandler + fmt::Debug>(
    ctx: &mut Context<T, Message>,
    session: &SessionClient,
    id: i64,
) -> Result<LoginRequest> {
    let (qrcode_id, data) = get_qrcode_id(session).await?;

    {
        let qrcode_url =
            format!("https://ids.xmu.edu.cn/authserver/qrCode/getCode?uuid={qrcode_id}");

        let qrcode_login =
            format!("https://ids.xmu.edu.cn/authserver/qrCode/qrCodeLogin.do?uuid={qrcode_id}");

        ctx.send_message(
            MessageSend::new_message()
                .at(id.to_string())
                .text(format!("将为{id}登录：\n"))
                .text("请使用企业微信扫码登录")
                .image_url(qrcode_url)
                .text("\n或者直接点击链接登录：")
                .text(qrcode_login)
                .build(),
        )
        .await?;
    }

    wait_qrcode(session, &qrcode_id).await?;

    Ok(data)
}

pub async fn process_login<T: BotClient + BotHandler + fmt::Debug>(
    ctx: &mut Context<T, Message>,
    id: i64,
) -> Result<()> {
    let session = SessionClient::new();

    let data = send_msg_and_wait(ctx, &session, id).await?;

    ctx.send_message_async(message::from_str("登录成功！"));

    let zzy_profile = update_db_and_login_base(&session, data, id).await?;

    ctx.send_message_async(message::from_str(format!(
        "信息:{} 转入学院:{:?}",
        zzy_profile.entry_year, zzy_profile.trans_dept
    )));

    let year = zzy_profile
        .entry_year
        .chars()
        .skip(2) // 跳过 "20"
        .take(2) // 取 "24"
        .collect::<String>();

    let dept = zzy_profile.trans_dept.join(",");

    ctx.set_title(format!("{}转{}", year, dept)).await?;

    Ok(())
}
