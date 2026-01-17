use crate::{
    abi::message::MessageSend,
    api::llm::chat::{
        archive::message_storage::{MessageStorage, MessageStorageExt, NoticeStorage},
        audit::bridge::llm_msg_from_message,
        llm::ask_llm,
    },
    config::LLM_AUDIT_DURATION_SECS,
};
use anyhow::Result;
use genai::chat::ChatMessage;
use std::time::{self, Duration};
use tokio::time::sleep;

pub async fn audit_test(message: &MessageSend) -> Result<()> {
    let ts = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let src_msg: Vec<ChatMessage> = llm_msg_from_message(message).await;
    tokio::spawn(async move {
        sleep(Duration::from_secs(LLM_AUDIT_DURATION_SECS + 5)).await;
        let before_msg = MessageStorage::get_range(ts - LLM_AUDIT_DURATION_SECS, ts).await;
        let before_notice = NoticeStorage::get_range(ts - LLM_AUDIT_DURATION_SECS, ts).await;
        let after_msg = MessageStorage::get_range(ts, ts + LLM_AUDIT_DURATION_SECS).await;
        let after_notice = NoticeStorage::get_range(ts, ts + LLM_AUDIT_DURATION_SECS).await;
        let mut msg = Vec::with_capacity(
            before_msg.len() + before_notice.len() + after_msg.len() + after_notice.len() + 10,
        );

        msg.push(ChatMessage::system(
            r#"# Role
你是一名高级对话审计员（Dialogue Auditor），负责评估 AI 助手在群聊环境中的回复质量。

# Context
你将获得三个部分的信息：
1. 历史背景（Context）：Bot 回复前的几条消息。
2. 待审计回复（Target）：Bot 实际发出的回复。
3. 用户反馈（User Feedback）：Bot 回复后 20s-60s 内其他用户的反应。

# Audit Criteria
请根据以下维度评估 [Target] 的表现：
- 语义一致性 (Semantic Consistency)：回复是否符合 [Context] 的讨论主题？是否存在严重的语义偏移（Drift）？
- 事实准确性 (Accuracy)：回复是否包含明显的事实错误，尤其是被 [User Feedback] 纠正的内容？
- 交互效果 (Engagement)：[User Feedback] 是否表现出困惑、反感、或明确的否定（如“你在说什么”、“错了吧”）？"#,
        ));

        msg.push(ChatMessage::system("助手回复消息"));
        msg.extend(src_msg);
        msg.push(ChatMessage::system("前60s的消息和提示"));
        msg.extend(before_msg);
        msg.extend(before_notice);
        msg.push(ChatMessage::system("后60s的消息和提示"));
        msg.extend(after_msg);
        msg.extend(after_notice);

        let audit_response = ask_llm(msg).await?;

        //TODO:验证回测结果
        todo!("对回测结果进行验证");

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}
