pub mod api;
pub mod event_body;
pub mod file;
pub mod helper;
pub mod message_body;
pub mod sender;

use crate::abi::message::message_body::*;

pub use api::Params;
pub use event_body as event;
pub use event_body::Event;
pub use event_body::message as event_message;
pub use event_body::meta as event_meta;
pub use event_body::notice as event_notice;
pub use event_body::request as event_request;
pub use event_body::{MessageType, Target, Type};
pub use helper::*;
pub use message_body::MessageReceive;
pub use message_body::MessageSend;
pub use sender::{Sender, SenderGroup, SenderPrivate};

pub fn from_str<S: Into<String>>(s: S) -> MessageSend {
    MessageSend::new_message().text(s).build()
}

fn receive_seq_to_send_seq(seq: &SegmentReceive) -> SegmentSend {
    match seq {
        SegmentReceive::Anonymous(_) => {
            SegmentSend::Anonymous(message_body::anonymous::DataSend { ignore: None })
        }
        SegmentReceive::At(p) => SegmentSend::At(p.clone()),
        SegmentReceive::Contact(p) => SegmentSend::Contact(p.clone()),
        SegmentReceive::Dice(p) => SegmentSend::Dice(p.clone()),
        SegmentReceive::Face(p) => SegmentSend::Face(p.clone()),
        SegmentReceive::Forward(p) => SegmentSend::Node(Box::new(
            message_body::node::DataSend::Id(message_body::node::DataSend1 { id: p.id.clone() }),
        )),
        SegmentReceive::Image(p) => SegmentSend::Image(Box::new(message_body::image::DataSend {
            file: p.url.clone(),
            r#type: p.r#type.clone(),
            cache: message_body::Cache::default(),
            proxy: message_body::Proxy::default(),
            timeout: None,
        })),
        SegmentReceive::Json(p) => SegmentSend::Json(p.clone()),
        SegmentReceive::Location(p) => {
            SegmentSend::Location(Box::new(message_body::location::DataSend {
                lat: p.lat.clone(),
                lon: p.lon.clone(),
                title: Some(p.title.clone()),
                content: Some(p.content.clone()),
            }))
        }
        SegmentReceive::Music(p) => SegmentSend::Music(p.clone()),
        SegmentReceive::Node() => SegmentSend::Node(Box::new(
            message_body::node::DataSend::Content(message_body::node::DataSend2 {
                user_id: "".to_string(),
                nickname: "".to_string(),
                content: Box::new(MessageSend::Array(Vec::new())),
            }),
        )),
        SegmentReceive::Poke(p) => SegmentSend::Poke(Box::new(message_body::poke::DataSend {
            r#type: p.r#type.clone(),
            id: p.id.clone(),
        })),
        SegmentReceive::Record(p) => {
            SegmentSend::Record(Box::new(message_body::record::DataSend {
                file: p.url.clone(),
                magic: message_body::Magic::default(),
                cache: message_body::Cache::default(),
                proxy: message_body::Proxy::default(),
                timeout: None,
            }))
        }
        SegmentReceive::Reply(p) => SegmentSend::Reply(p.clone()),
        SegmentReceive::Rps(p) => SegmentSend::Rps(p.clone()),
        SegmentReceive::Shake(p) => SegmentSend::Shake(p.clone()),
        SegmentReceive::Share(p) => SegmentSend::Share(Box::new(message_body::share::DataSend {
            url: p.url.clone(),
            title: p.title.clone(),
            content: Some(p.content.clone()),
            image: Some(p.image.clone()),
        })),
        SegmentReceive::Text(p) => SegmentSend::Text(p.clone()),
        SegmentReceive::Video(p) => SegmentSend::Video(Box::new(message_body::video::DataSend {
            file: p.url.clone(),
            cache: message_body::Cache::default(),
            proxy: message_body::Proxy::default(),
            timeout: None,
        })),
        SegmentReceive::Xml(p) => SegmentSend::Xml(p.clone()),
    }
}

pub fn receive2send(msg: &MessageReceive) -> MessageSend {
    let msg_vec = match msg {
        MessageReceive::Array(arr) => arr.iter().map(receive_seq_to_send_seq).collect::<Vec<_>>(),
        MessageReceive::Single(sing) => {
            vec![receive_seq_to_send_seq(sing)]
        }
    };

    MessageSend::Array(msg_vec)
}

pub fn receive2send_add_prefix(msg: &MessageReceive, prefix: String) -> MessageSend {
    let mut msg_vec = match msg {
        MessageReceive::Array(arr) => arr.iter().map(receive_seq_to_send_seq).collect::<Vec<_>>(),
        MessageReceive::Single(sing) => {
            vec![receive_seq_to_send_seq(sing)]
        }
    };

    let mut ret = vec![SegmentSend::Text(Box::new(message_body::text::Data {
        text: prefix,
    }))];

    ret.append(&mut msg_vec);

    MessageSend::Array(ret)
}
