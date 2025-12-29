use helper::define_default_type;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CQ {
    pub user_id: u64,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SegmentReceive {
    Text(text::Data),
    Face(face::Data),
    Image(image::DataReceive),
    Record(record::DataReceive),
    Video(video::DataReceive),
    At(at::Data),
    Rps(rps::Data),
    Dice(dice::Data),
    Shake(shake::Data),
    Poke(poke::DataReceive),
    Anonymous(anonymous::DataReceive),
    Music(music::Data),
    Share(share::DataReceive),
    Contact(contact::Data),
    Location(location::DataReceive),
    Reply(reply::Data),
    Forward(Box<forward::DataReceive>),
    Node(),
    Xml(xml::Data),
    Json(json::Data),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SegmentSend {
    Text(text::Data),
    Face(face::Data),
    Image(image::DataSend),
    Record(record::DataSend),
    Video(video::DataSend),
    At(at::Data),
    Rps(rps::Data),
    Dice(dice::Data),
    Shake(shake::Data),
    Poke(poke::DataSend),
    Anonymous(anonymous::DataSend),
    Music(music::Data),
    Share(share::DataSend),
    Contact(contact::Data),
    Location(location::DataSend),
    Reply(reply::Data),
    Forward(),
    Node(Box<node::DataSend>),
    Xml(xml::Data),
    Json(json::Data),
}

type ArraySend = Vec<SegmentSend>;
type ArrayReceive = Vec<SegmentReceive>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageReceive {
    Array(ArrayReceive),
    Single(SegmentReceive),
    //Cq(CQ),
}

impl MessageReceive {
    pub fn get_text(&self) -> String {
        match self {
            //MessageReceive::Cq(cq) => cq.message.clone(),
            MessageReceive::Single(seg) => match seg {
                SegmentReceive::Text(data) => data.text.clone(),
                _ => String::new(),
            },
            MessageReceive::Array(arr) => {
                let capacity = arr
                    .iter()
                    .filter_map(|seg| {
                        if let SegmentReceive::Text(data) = seg {
                            Some(data.text.len())
                        } else {
                            None
                        }
                    })
                    .sum();

                let mut result = String::with_capacity(capacity);
                for seg in arr {
                    if let SegmentReceive::Text(data) = seg {
                        result.push_str(&data.text);
                    }
                }
                result
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageSend {
    Array(ArraySend),
    Single(SegmentSend),
    Cq(CQ),
}

pub mod text {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Data {
        pub text: String,
    }
}

pub mod face {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Data {
        pub id: String,
    }
}

define_default_type!(Cache, u8, 1);
define_default_type!(Proxy, u8, 1);

pub mod image {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataReceive {
        pub file: String,
        pub r#type: Option<String>,
        pub url: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataSend {
        pub file: String,
        pub r#type: Option<String>,
        pub cache: Cache,
        pub proxy: Proxy,
        pub timeout: Option<u64>,
    }
}

define_default_type!(Magic, u8, 0);

pub mod record {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataReceive {
        pub file: String,
        pub magic: Magic,
        pub url: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataSend {
        pub file: String,
        pub magic: Magic,
        pub cache: Cache,
        pub proxy: Proxy,
        pub timeout: Option<u64>,
    }
}

pub mod video {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataReceive {
        pub file: String,
        pub url: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataSend {
        pub file: String,
        pub cache: Cache,
        pub proxy: Proxy,
        pub timeout: Option<u64>,
    }
}

pub mod at {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Data {
        pub qq: String,
    }
}

pub mod rps {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Data {}
}

pub mod dice {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Data {}
}

pub mod shake {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Data {}
}

pub mod poke {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataReceive {
        pub r#type: String,
        pub id: String,
        pub name: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataSend {
        pub r#type: String,
        pub id: String,
    }
}

pub mod anonymous {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataReceive {}

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataSend {
        pub ignore: Option<u8>,
    }
}

pub mod share {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataReceive {
        pub url: String,
        pub title: String,
        pub content: String,
        pub image: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataSend {
        pub url: String,
        pub title: String,
        pub content: Option<String>,
        pub image: Option<String>,
    }
}

pub mod contact {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Data {
        Qq { id: String },
        Group { id: String },
    }
}

pub mod location {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataReceive {
        pub lat: String,
        pub lon: String,
        pub title: String,
        pub content: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataSend {
        pub lat: String,
        pub lon: String,
        pub title: Option<String>,
        pub content: Option<String>,
    }
}

pub mod music {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataReceive {}

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Data {
        Qq {
            id: String,
        },
        #[serde(rename = "163")]
        NetEase163 {
            id: String,
        },
        Xm {
            id: String,
        },
        Custom {
            url: String,
            audio: String,
            title: String,
            content: Option<String>,
            image: Option<String>,
        },
    }
}

pub mod reply {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Data {
        pub id: String,
    }
}

pub mod forward {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataReceive {
        pub id: String,
    }
}

pub mod node {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(untagged)]
    pub enum DataSend {
        Id(DataSend1),
        Content(DataSend2),
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataSend1 {
        pub id: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DataSend2 {
        pub user_id: String,
        pub nickname: String,
        pub content: Box<MessageSend>,
    }
}

pub mod xml {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Data {
        pub data: String,
    }
}

pub mod json {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Data {
        pub data: String,
    }
}
