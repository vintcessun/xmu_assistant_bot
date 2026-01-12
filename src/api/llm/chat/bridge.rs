use crate::{
    abi::message::{MessageReceive, message_body::SegmentReceive},
    api::llm::chat::identity::search_identity,
};
use genai::chat::{Binary, ChatMessage, ContentPart};

include!(concat!(env!("OUT_DIR"), "/face_data.rs"));

pub fn get_gif_from_exe(id: &str) -> Option<(&'static str, &'static str, &'static str)> {
    FACES.get(id).copied()
}

async fn llm_msg_from_segment_receive(segment: &SegmentReceive) -> ChatMessage {
    match segment {
        SegmentReceive::Text(e) => ChatMessage::user(e.text),
        SegmentReceive::Face(e) => {
            let id: String = e.id.into();
            let gif_data = get_gif_from_exe(&id);
            match gif_data {
                Some(data) => {
                    let (content_type, content, name) = data;
                    let parts = vec![
                        ContentPart::Text(format!("[face: {}]", id)),
                        ContentPart::Binary(Binary::from_base64(
                            content_type,
                            content,
                            Some(name.to_string()),
                        )),
                    ];
                    ChatMessage::user(parts)
                }
                None => ChatMessage::user(format!("[Unknown Face: {}]", id)),
            }
        }
        SegmentReceive::Image(e) => ChatMessage::user(ContentPart::Binary(Binary::from_url(
            e.r#type.into(),
            e.url.into(),
            Some(e.file.into()),
        ))),
        SegmentReceive::Record(e) => ChatMessage::user(ContentPart::Binary(Binary::from_url(
            "audio/amr",
            e.url.into(),
            Some(e.file.into()),
        ))),
        SegmentReceive::Video(e) => ChatMessage::user(ContentPart::Binary(Binary::from_url(
            "video/mp4",
            e.url.into(),
            Some(e.file.into()),
        ))),
        SegmentReceive::At(e) => {
            let user_id: String = e.qq.into();
            let qq_i64 = user_id.parse::<i64>().unwrap_or(0);
            let identity = search_identity(qq_i64).await;
            let identity_data = match identity {
                Some(data) => quick_xml::se::to_string(&identity).unwrap_or("未知身份".to_string()),
                None => "未知身份".to_string(),
            };
            ChatMessage::user(format!("[At {user_id}]<data>{identity_data}</data>"))
        }
        SegmentReceive::Rps(_) => ChatMessage::user("[RPS 猜拳魔法表情]"),
        SegmentReceive::Dice(_) => ChatMessage::user("[Dice 掷骰子魔法表情]"),
        SegmentReceive::Shake(_) => ChatMessage::user("[Shake 窗口抖动（戳一戳）]"),
        SegmentReceive::Poke(e) => {}
    }
}

pub fn llm_msg_from_message_receive(message: &MessageReceive) -> Vec<ChatMessage> {
    match message {
        MessageReceive::Array(e) => e
            .iter()
            .map(|seg| llm_msg_from_segment_receive(seg))
            .collect(),
        MessageReceive::Single(e) => {
            vec![llm_msg_from_segment_receive(e)]
        }
    }
}
