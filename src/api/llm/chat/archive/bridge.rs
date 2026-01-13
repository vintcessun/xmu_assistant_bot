use crate::{
    abi::{
        logic_import::{Message, Notice},
        message::{
            MessageReceive,
            message_body::{SegmentReceive, contact},
        },
    },
    api::llm::chat::archive::{
        identity::{IdentityGroup, IdentityPerson},
        message_storage::MessageStorage,
    },
};
use genai::chat::{Binary, ChatMessage, ContentPart, MessageContent};

include!(concat!(env!("OUT_DIR"), "/face_data.rs"));

pub fn get_gif_from_exe(id: &str) -> Option<(&'static str, &'static str, &'static str)> {
    FACES.get(id).copied()
}

async fn llm_msg_from_segment_receive(segment: &SegmentReceive) -> ChatMessage {
    match segment {
        SegmentReceive::Text(e) => ChatMessage::user(e.text.get()),
        SegmentReceive::Face(e) => {
            let id = e.id.get();
            let gif_data = get_gif_from_exe(id);
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
        SegmentReceive::Image(e) => {
            let content_type = match &e.r#type {
                Some(t) => t.get(),
                None => "image/jpeg",
            };
            let url = e.url.get();
            ChatMessage::user(ContentPart::Binary(Binary::from_url(
                content_type,
                url,
                Some(e.file.get().to_string()),
            )))
        }
        SegmentReceive::Record(e) => ChatMessage::user(ContentPart::Binary(Binary::from_url(
            "audio/amr",
            e.url.get(),
            Some(e.file.get().to_string()),
        ))),
        SegmentReceive::Video(e) => ChatMessage::user(ContentPart::Binary(Binary::from_url(
            "video/mp4",
            e.url.get(),
            Some(e.file.get().to_string()),
        ))),
        SegmentReceive::At(e) => {
            let user_id = e.qq.get();
            let qq_i64 = user_id.parse::<i64>().unwrap_or(0);
            let identity = IdentityPerson::get(qq_i64).await;
            let identity_data = match identity {
                Some(data) => quick_xml::se::to_string(&data).unwrap_or("未知身份".to_string()),
                None => "未知身份".to_string(),
            };
            ChatMessage::user(format!("[At {user_id}]<data>{identity_data}</data>"))
        }
        SegmentReceive::Rps(_) => ChatMessage::user("[RPS 猜拳魔法表情]"),
        SegmentReceive::Dice(_) => ChatMessage::user("[Dice 掷骰子魔法表情]"),
        SegmentReceive::Poke(e) => {
            let type_id = e.r#type.get();
            let id = e.id.get();
            let name = e.name.get();

            ChatMessage::user(format!(
                "[戳一戳消息, ID: ({},{}), 名称: {}]",
                type_id, id, name
            ))
        }
        SegmentReceive::Share(e) => {
            let content = vec![
                ContentPart::Text(format!(
                    "[分享链接 标题: {} 链接: {} 内容: {}]",
                    e.title.get(),
                    e.url.get(),
                    e.content.get()
                )),
                ContentPart::Binary(Binary::from_url(
                    "image/jpeg",
                    e.image.get(),
                    Some(e.title.get().to_string()),
                )),
            ];
            ChatMessage::user(content)
        }
        SegmentReceive::Contact(e) => match e {
            contact::DataReceive::Group(g) => {
                let group_id = g.id.get();
                let group_i64 = group_id.parse::<i64>().unwrap_or(0);
                let identity = IdentityGroup::get(group_i64);
                let identity_data = match identity.await {
                    Some(data) => {
                        quick_xml::se::to_string(&data).unwrap_or("未知群身份".to_string())
                    }
                    None => "未知群身份".to_string(),
                };
                ChatMessage::user(format!(
                    "[推荐群聊 {}]<data>{}</data>",
                    group_id, identity_data
                ))
            }
            contact::DataReceive::Qq(q) => {
                let qq = q.id.get();
                let qq_i64 = qq.parse::<i64>().unwrap_or(0);
                let identity = IdentityPerson::get(qq_i64);
                let identity_data = match identity.await {
                    Some(data) => {
                        quick_xml::se::to_string(&data).unwrap_or("未知群身份".to_string())
                    }
                    None => "未知群身份".to_string(),
                };
                ChatMessage::user(format!("[推荐群聊 {}]<data>{}</data>", qq, identity_data))
            }
        },
        SegmentReceive::Location(e) => ChatMessage::user(format!(
            "[位置 {} 内容: {} 经度: {} 纬度: {}]",
            e.title.get(),
            e.content.get(),
            e.lon.get(),
            e.lat.get(),
        )),
        SegmentReceive::Reply(e) => {
            let msg_id = e.id.get();
            let content = vec![ContentPart::Text(format!("[回复消息 ID: {}]", msg_id))];
            let msg_content = match MessageStorage::get(msg_id.to_string()).await {
                Some(c) => {
                    let mut content = MessageContent::from(content);
                    content.extend(c.content);
                    content
                }
                None => MessageContent::from(content),
            };
            ChatMessage::user(msg_content)
        }
        SegmentReceive::Forward(e) => {
            let id = e.id.get();
            let content = vec![ContentPart::Text(format!("[转发消息 id: {id}]"))];

            let msg = MessageStorage::get(id.to_string()).await;

            let msg_content = match msg {
                Some(e) => {
                    let mut content = MessageContent::from(content);
                    content.extend(e.content);
                    content
                }
                None => MessageContent::from(content),
            };
            ChatMessage::user(msg_content)
        }
        SegmentReceive::Xml(e) => ChatMessage::user(format!("[XML消息 {}]", e.data.get())),
        SegmentReceive::Json(e) => ChatMessage::user(format!("[JSON消息 {}]", e.data.get())),
    }
}

pub async fn llm_msg_from_message_receive(message: &MessageReceive) -> Vec<ChatMessage> {
    match message {
        MessageReceive::Array(e) => {
            let mut result = Vec::with_capacity(e.len());
            for seg in e.iter() {
                result.push(llm_msg_from_segment_receive(seg).await);
            }
            result
        }
        MessageReceive::Single(e) => {
            vec![llm_msg_from_segment_receive(e).await]
        }
    }
}

pub async fn llm_msg_from_message(message: &Message) -> Vec<ChatMessage> {
    match message {
        Message::Private(p) => {
            let data = quick_xml::se::to_string(&p).unwrap_or("未知消息".to_string());
            let mut ret = vec![ChatMessage::user(format!("<data>{}</data>", data))];
            ret.extend(llm_msg_from_message_receive(&p.message).await);
            ret
        }
        Message::Group(g) => {
            let data = quick_xml::se::to_string(&g).unwrap_or("未知消息".to_string());
            let mut ret = vec![ChatMessage::user(format!("<data>{}</data>", data))];
            ret.extend(llm_msg_from_message_receive(&g.message).await);
            ret
        }
    }
}

pub async fn llm_msg_from_notice(notice: &Notice) -> ChatMessage {
    ChatMessage::user(quick_xml::se::to_string(notice).unwrap_or("未知提示".to_string()))
}
