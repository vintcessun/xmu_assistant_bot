pub mod api;
pub mod event_body;
mod file;
mod message_body;
mod sender;

pub use api::Params;
pub use event_body as event;
pub use event_body::Event;
pub use event_body::message as event_message;
pub use event_body::meta as event_meta;
pub use event_body::notice as event_notice;
pub use event_body::request as event_request;
pub use event_body::{MessageType, Target};
pub use message_body::MessageReceive;
pub use message_body::MessageSend;
pub use sender::{Sender, SenderGroup, SenderPrivate};

pub fn from_str(s: String) -> MessageSend {
    MessageSend::Single(message_body::SegmentSend::Text(message_body::text::Data {
        text: s,
    }))
}
