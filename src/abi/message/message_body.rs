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
    Text(Box<text::Data>),
    Face(Box<face::Data>),
    Image(Box<image::DataReceive>),
    Record(Box<record::DataReceive>),
    Video(Box<video::DataReceive>),
    At(Box<at::Data>),
    Rps(rps::Data),
    Dice(dice::Data),
    Shake(shake::Data),
    Poke(Box<poke::DataReceive>),
    Anonymous(anonymous::DataReceive),
    Music(Box<music::Data>),
    Share(Box<share::DataReceive>),
    Contact(Box<contact::Data>),
    Location(Box<location::DataReceive>),
    Reply(Box<reply::Data>),
    Forward(Box<forward::DataReceive>),
    Node(),
    Xml(Box<xml::Data>),
    Json(Box<json::Data>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SegmentSend {
    Text(Box<text::Data>),
    Face(Box<face::Data>),
    Image(Box<image::DataSend>),
    Record(Box<record::DataSend>),
    Video(Box<video::DataSend>),
    At(Box<at::Data>),
    Rps(rps::Data),
    Dice(dice::Data),
    Shake(shake::Data),
    Poke(Box<poke::DataSend>),
    Anonymous(anonymous::DataSend),
    Music(Box<music::Data>),
    Share(Box<share::DataSend>),
    Contact(Box<contact::Data>),
    Location(Box<location::DataSend>),
    Reply(Box<reply::Data>),
    Forward(),
    Node(Box<node::DataSend>),
    Xml(Box<xml::Data>),
    Json(Box<json::Data>),
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
            // 1. 极速路径：单条文本直接 Clone
            MessageReceive::Single(SegmentReceive::Text(data)) => data.text.clone(),

            // 2. 数组路径：利用 Extend 内部优化
            MessageReceive::Array(arr) => {
                let mut result = String::new();
                result.extend(arr.iter().filter_map(|seg| {
                    if let SegmentReceive::Text(data) = seg {
                        Some(data.text.as_str())
                    } else {
                        None
                    }
                }));
                result
            }

            // 3. 其他情况：统一返回空 String (Inline 处理)
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_size() {
        println!("\n========================================");
        println!("MessageSend size: {} bytes", size_of::<MessageSend>());
        println!("MessageReceive size: {} bytes", size_of::<MessageReceive>());
        println!("========================================\n\n");
    }

    #[test]
    fn locate_large_variants() {
        let mut sizes = vec![
            ("Text", size_of::<text::Data>()),
            ("Face", size_of::<face::Data>()),
            ("Image", size_of::<image::DataSend>()),
            ("Record", size_of::<record::DataSend>()),
            ("Video", size_of::<video::DataSend>()),
            ("At", size_of::<at::Data>()),
            ("Rps", size_of::<rps::Data>()),
            ("Dice", size_of::<dice::Data>()),
            ("Shake", size_of::<shake::Data>()),
            ("Poke", size_of::<poke::DataSend>()),
            ("Anonymous", size_of::<anonymous::DataSend>()),
            ("Music", size_of::<music::Data>()),
            ("Share", size_of::<share::DataSend>()),
            ("Contact", size_of::<contact::Data>()),
            ("Location", size_of::<location::DataSend>()),
            ("Reply", size_of::<reply::Data>()),
            ("Xml", size_of::<xml::Data>()),
            ("Json", size_of::<json::Data>()),
            ("Node", size_of::<node::DataSend>()),
        ];

        // 按字节大小降序排列
        sizes.sort_by(|a, b| b.1.cmp(&a.1));

        println!("\n========================================");
        println!("      SEGMENT VARIANT SIZE ANALYSIS      ");
        println!("========================================");
        println!("Total SegmentSend size: {} bytes", size_of::<SegmentSend>());
        println!("Total MessageSend size: {} bytes", size_of::<MessageSend>());
        println!("----------------------------------------");

        for (name, size) in sizes {
            let status = if size > 64 {
                "!! [REALLY LARGE]"
            } else if size > 32 {
                "!  [LARGE]"
            } else {
                "   [OK]"
            };
            println!("{:<20} : {:>3} bytes  {}", name, size, status);
        }
        println!("========================================\n");
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageSend {
    Array(ArraySend),
    Single(SegmentSend),
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
