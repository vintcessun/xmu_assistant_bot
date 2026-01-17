use crate::{
    abi::{Context, message::MessageType, network::BotClient, websocket::BotHandler},
    api::llm::chat::repeat::reply::{MessageAbstract, get_repeat_reply},
};
use anyhow::{Result, anyhow};

pub async fn send_message_from_hot<T, M>(ctx: &mut Context<T, M>) -> Result<()>
where
    T: BotClient + BotHandler + std::fmt::Debug + 'static,
    M: MessageType + std::fmt::Debug + Send + Sync + 'static,
{
    let msg = ctx.get_message_text().to_string();
    let sender = ctx
        .get_message()
        .get_sender()
        .user_id
        .ok_or(anyhow!("查询用户失败"))?;
    let message = MessageAbstract {
        qq: sender,
        msg_text: msg,
    };

    let message_send = get_repeat_reply(message)
        .await
        .ok_or(anyhow!("未命中热回复"))?;

    ctx.send_message(message_send).await?;

    //TODO:完成回测
    todo!("进行回测");

    Ok(())
}
