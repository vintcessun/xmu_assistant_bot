use std::sync::LazyLock;

use super::process::process_login;
use crate::api::storage::HotTable;
use crate::api::xmu_service::login::LoginData;
use crate::{abi::logic_import::*, api::xmu_service::lnt::Profile};
use anyhow::anyhow;

pub static DATA: LazyLock<HotTable<i64, LoginData>> = LazyLock::new(|| HotTable::new("login"));

#[handler(msg_type=Message,command="login",echo_cmd=true)]
pub async fn login(ctx: Context) -> Result<()> {
    let sender = ctx.message.get_sender();
    let id = sender.user_id.ok_or(anyhow!("获取用户ID失败"))?;

    match DATA.get(&id) {
        Some(e) => {
            if Profile::check(&e.lnt).await {
                ctx.send_message_async(message::from_str("已登录，请用其他命令查询"));
            } else {
                ctx.send_message_async(message::from_str("登录信息失效"));
                process_login(&mut ctx, id).await?;
            }
        }

        None => {
            process_login(&mut ctx, id).await?;
        }
    }

    Ok(())
}

#[handler(msg_type=Message,command="logout",echo_cmd=true)]
pub async fn logout(ctx: Context) -> Result<()> {
    let sender = ctx.message.get_sender();
    let id = sender.user_id.ok_or(anyhow!("获取用户ID失败"))?;

    DATA.remove(&id)?;

    ctx.send_message_async(message::from_str("已删除登录数据"));

    Ok(())
}
