use crate::{
    abi::{
        Context,
        logic_import::Message,
        message::{MessageType, Target},
        network::BotClient,
        websocket::BotHandler,
    },
    api::llm::chat::{
        audit::audit_test_fast,
        repeat::reply::{MessageAbstract, RepeatReply},
    },
};
use anyhow::{Result, anyhow};

pub async fn send_message_from_store<T, M>(ctx: &mut Context<T, Message>) -> Result<()>
where
    T: BotClient + BotHandler + std::fmt::Debug + 'static,
{
    let message = ctx.get_message();
    let id = match &*message {
        Message::Group(g) => g.message_id.to_string(),
        Message::Private(p) => p.message_id.to_string(),
    };

    //TODO: 从向量数据库中搜索得到回复内容
    todo!();

    //ctx.send_message(message_send).await?;

    Ok(())
}
