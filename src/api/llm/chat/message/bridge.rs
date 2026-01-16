use crate::{
    abi::message::{
        MessageSend,
        file::FileUrl,
        message_body::{Cache, Proxy, SegmentSend, at, face, image, text},
    },
    api::{
        llm::{
            chat::{archive::bridge::get_face_reference_message, file::LlmFile},
            tool::{LlmPrompt, LlmVec, ask_as},
        },
        storage::FileStorage,
    },
};
use anyhow::Result;
use genai::chat::{ChatMessage, ChatResponse};
use helper::{LlmPrompt, box_new};
use serde::{Deserialize, Serialize};

#[derive(Debug, LlmPrompt, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SegmentSendLlmResponse {
    #[prompt("纯文本内容")]
    Text {
        #[prompt("文本内容")]
        text: String,
    },

    #[prompt("图片内容")]
    Image {
        #[prompt("图片文件")]
        file: LlmFile,
    },

    #[prompt("QQ表情")]
    Face {
        #[prompt("表情ID")]
        id: String,
    },

    #[prompt("提及某人")]
    At {
        #[prompt("提及对象的QQ号")]
        qq: String,
    },
}

#[derive(Debug, LlmPrompt, Serialize, Deserialize)]
pub struct MessageSendLlmResponse {
    #[prompt("请根据提供的回复改写并运用提供的符号体系进行回应")]
    pub message: LlmVec<SegmentSendLlmResponse>,
}

pub struct IntoMessageSend;

impl IntoMessageSend {
    pub async fn get(msg: ChatResponse) -> Result<MessageSendLlmResponse> {
        let messages = vec![
            ChatMessage::system(
                "你是一个专业的将消息进行转写的助手，请根据用户提供的信息和所有上下文进行转写为规范格式",
            ),
            get_face_reference_message(),
            ChatMessage::user(msg.content),
        ];

        let response = ask_as::<MessageSendLlmResponse>(messages).await?;
        Ok(response)
    }

    pub async fn get_message_send(msg: ChatResponse) -> Result<MessageSend> {
        let msg = Self::get(msg).await?;
        let mut ret = Vec::with_capacity(msg.message.len());
        for segment in msg.message {
            ret.push(match segment {
                SegmentSendLlmResponse::At { qq } => SegmentSend::At(at::DataSend { qq }),
                SegmentSendLlmResponse::Face { id } => SegmentSend::Face(face::DataSend { id }),
                SegmentSendLlmResponse::Image { file } => {
                    SegmentSend::Image(box_new!(image::DataSend, {
                        file: FileUrl::from_path(file.file.get_path())?,
                        r#type: None,
                        cache: Cache::default(),
                        proxy: Proxy::default(),
                        timeout: None,
                    }))
                }
                SegmentSendLlmResponse::Text { text } => SegmentSend::Text(text::DataSend { text }),
            })
        }
        Ok(MessageSend::Array(ret))
    }
}
