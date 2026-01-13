pub mod api;
pub mod event_body;
pub mod file;
pub mod helper;
pub mod message_body;
pub mod sender;

use crate::abi::message::message_body::*;
use crate::box_new;

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
        SegmentReceive::At(p) => SegmentSend::At(message_body::at::DataSend {
            qq: p.qq.clone().into(),
        }),
        SegmentReceive::Contact(p) => {
            let val = match p {
                message_body::contact::DataReceive::Qq(e) => {
                    message_body::contact::DataSend::Qq(message_body::contact::QqSend {
                        id: e.id.clone().into(),
                    })
                }
                message_body::contact::DataReceive::Group(e) => {
                    message_body::contact::DataSend::Group(message_body::contact::GroupSend {
                        id: e.id.clone().into(),
                    })
                }
            };
            SegmentSend::Contact(val)
        }
        SegmentReceive::Dice(p) => SegmentSend::Dice(p.clone()),
        SegmentReceive::Face(p) => {
            let val = message_body::face::DataSend {
                id: p.id.clone().into(),
            };
            SegmentSend::Face(val)
        }
        SegmentReceive::Forward(p) => SegmentSend::Node(message_body::node::DataSend::Id(
            message_body::node::DataSend1 {
                id: p.id.clone().into(),
            },
        )),
        SegmentReceive::Image(p) => SegmentSend::Image(box_new!(message_body::image::DataSend, {
            file: file::FileUrl::new(p.url.clone().into()),
            r#type: LazyString::into_opt_string(p.r#type.clone()),
            cache: message_body::Cache::default(),
            proxy: message_body::Proxy::default(),
            timeout: None,
        })),
        SegmentReceive::Json(p) => SegmentSend::Json(message_body::json::DataSend {
            data: p.data.clone().into(),
        }),
        SegmentReceive::Location(p) => {
            SegmentSend::Location(box_new!(message_body::location::DataSend, {
                lat: p.lat.clone().into(),
                lon: p.lon.clone().into(),
                title: Some(p.title.clone().into()),
                content: Some(p.content.clone().into()),
            }))
        }
        SegmentReceive::Poke(p) => SegmentSend::Poke(message_body::poke::DataSend {
            r#type: p.r#type.clone().into(),
            id: p.id.clone().into(),
        }),
        SegmentReceive::Record(p) => SegmentSend::Record(message_body::record::DataSend {
            file: file::FileUrl::new(p.url.clone().into()),
            magic: message_body::Magic::default(),
            cache: message_body::Cache::default(),
            proxy: message_body::Proxy::default(),
            timeout: None,
        }),
        SegmentReceive::Reply(p) => SegmentSend::Reply(message_body::reply::DataSend {
            id: p.id.clone().into(),
        }),
        SegmentReceive::Rps(p) => SegmentSend::Rps(p.clone()),
        SegmentReceive::Share(p) => SegmentSend::Share(box_new!(message_body::share::DataSend, {
            url: p.url.clone().into(),
            title: p.title.clone().into(),
            content: Some(p.content.clone().into()),
            image: Some(p.image.clone().into()),
        })),
        SegmentReceive::Text(p) => SegmentSend::Text(message_body::text::DataSend {
            text: p.text.clone().into(),
        }),
        SegmentReceive::Video(p) => SegmentSend::Video(message_body::video::DataSend {
            file: file::FileUrl::new(p.url.clone().into()),
            cache: message_body::Cache::default(),
            proxy: message_body::Proxy::default(),
            timeout: None,
        }),
        SegmentReceive::Xml(p) => SegmentSend::Xml(message_body::xml::DataSend {
            data: p.data.clone().into(),
        }),
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

    let mut ret = vec![SegmentSend::Text(message_body::text::DataSend {
        text: prefix,
    })];

    ret.append(&mut msg_vec);

    MessageSend::Array(ret)
}
