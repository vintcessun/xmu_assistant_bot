use crate::{
    abi::{
        Context,
        logic_import::{Message, Notice},
        network::BotClient,
        websocket::BotHandler,
    },
    api::llm::chat::{
        archive::{
            identity_group_archive, identity_person_archive, message_archive, notice_archive,
        },
        repeat::send_message_from_hot,
    },
};

pub async fn handle_llm_message<T>(ctx: &mut Context<T, Message>)
where
    T: BotClient + BotHandler + std::fmt::Debug + Send + Sync + 'static,
{
    message_archive(ctx).await;
    identity_person_archive(ctx).await;
    identity_group_archive(ctx).await;

    //L0: 命中回复
    if send_message_from_hot(ctx).await.is_ok() {
        return;
    }

    //L1: 搜索回复
    todo!("搜索回复逻辑待实现");

    //L2: AI生成回复
    todo!("AI生成回复逻辑待实现");
}

pub async fn handle_llm_notice<T>(ctx: &mut Context<T, Notice>)
where
    T: BotClient + BotHandler + std::fmt::Debug + Send + Sync + 'static,
{
    notice_archive(ctx).await;
}
