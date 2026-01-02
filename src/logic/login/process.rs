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

    let profile = Profile::get_profile(&login_data.lnt).await?;

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
        let qrcode_url = format!(
            "https://ids.xmu.edu.cn/authserver/qrCode/getCode?uuid={}",
            qrcode_id
        );

        ctx.send_message(
            MessageSend::new_message()
                .at(id.to_string())
                .text("请使用企业微信扫码登录：")
                .image(qrcode_url)
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

    ctx.send_message_async(message::from_str("登录成功！"));

    let data = send_msg_and_wait(ctx, &session, id).await?;

    let zzy_profile = update_db_and_login_base(&session, data, id).await?;

    ctx.send_message_async(message::from_str(format!(
        "信息:{} 转入学院:{:?}",
        zzy_profile.entry_year, zzy_profile.trans_dept
    )));

    Ok(())
}
