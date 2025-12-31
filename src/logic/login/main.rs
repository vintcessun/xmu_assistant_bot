use super::process::process_login;
use crate::abi::logic_import::*;
use crate::api::storage::HotTable;
use anyhow::anyhow;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::format;

lazy_static! {
    pub static ref DATA: HotTable<i64, LoginData> = HotTable::new("login");
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginData {
    pub castgc: String,
    pub lnt: String,
}

#[handler(msg_type=Message,command="login",echo_cmd=true)]
pub async fn login(ctx: Context) -> Result<()> {
    let sender = ctx.message.get_sender();
    let id = sender.user_id.ok_or(anyhow!("获取用户ID失败"))?;

    match DATA.get(&id) {
        Some(e) => {
            ctx.send_message_async(message::from_str("你已经登录过了！校验逻辑还未实现"));
        }

        None => {
            let ret = process_login(&mut ctx, id).await;
            if ret.is_err() {
                ctx.send_message_async(message::from_str(format!(
                    "运行出现错误: {}",
                    ret.err().unwrap()
                )));
            }
        }
    }

    Ok(())
}
