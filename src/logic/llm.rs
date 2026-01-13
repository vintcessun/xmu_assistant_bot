use crate::{
    abi::logic_import::*,
    api::llm::chat::router::{handle_llm_message, handle_llm_notice},
};

#[handler(msg_type=Message)]
pub async fn llm_message(ctx: Context) -> Result<()> {
    handle_llm_message(&mut ctx).await;
    Ok(())
}

#[handler(msg_type=Notice)]
pub async fn llm_notice(ctx: Context) -> Result<()> {
    handle_llm_notice(&mut ctx).await;
    Ok(())
}
