use crate::abi::message::file;
use crate::box_new;

use super::MessageSend;
use super::message_body::*;

pub struct MessageSendBuilder {
    segments: Vec<SegmentSend>,
}

impl MessageSendBuilder {
    pub fn new() -> Self {
        Self {
            segments: Vec::with_capacity(4),
        }
    }
}

impl Default for MessageSendBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageSendBuilder {
    pub fn build(self) -> MessageSend {
        MessageSend::Array(self.segments)
    }

    pub fn add_seg(mut self, segment: SegmentSend) -> Self {
        self.segments.push(segment);
        self
    }

    pub fn add_vec(mut self, segments: Vec<SegmentSend>) -> Self {
        self.segments.reserve(segments.len());
        self.segments.extend(segments);
        self
    }

    pub fn add_arr(mut self, segments: &[SegmentSend]) -> Self {
        self.segments.reserve(segments.len());
        self.segments.extend_from_slice(segments);
        self
    }

    pub fn add_msg(mut self, message: MessageSend) -> Self {
        match message {
            MessageSend::Array(arr) => {
                self.segments.reserve(arr.len());
                self.segments.extend(arr);
            }
            MessageSend::Single(seg) => {
                self.segments.push(seg);
            }
        }
        self
    }

    pub fn text<S: Into<String>>(self, s: S) -> Self {
        self.add_seg(SegmentSend::Text(box_new!(text::Data, { text: s.into() })))
    }

    pub fn face<S: Into<String>>(self, id: S) -> Self {
        self.add_seg(SegmentSend::Face(box_new!(face::Data, { id: id.into() })))
    }

    pub fn image_url<S: Into<String>>(self, url: S) -> Self {
        self.image(file::FileUrl::Raw(url.into()))
    }

    pub fn image(self, url: file::FileUrl) -> Self {
        self.add_seg(SegmentSend::Image(box_new!(image::DataSend, {
            file: url,
            cache: Cache::default(),
            proxy: Proxy::default(),
            timeout: None,
            r#type: None,
        })))
    }

    pub fn flash_image_url<S: Into<String>>(self, url: S) -> Self {
        self.flash_image(file::FileUrl::Raw(url.into()))
    }

    pub fn flash_image(self, url: file::FileUrl) -> Self {
        self.add_seg(SegmentSend::Image(box_new!(image::DataSend, {
            file: url,
            cache: Cache::default(),
            proxy: Proxy::default(),
            timeout: None,
            r#type: Some("flash".to_string()),
        })))
    }

    pub fn record_url<S: Into<String>>(self, url: S) -> Self {
        self.record(file::FileUrl::Raw(url.into()))
    }

    pub fn record(self, url: file::FileUrl) -> Self {
        self.add_seg(SegmentSend::Record(box_new!(record::DataSend, {
            file: url,
            magic: Magic::default(),
            cache: Cache::default(),
            proxy: Proxy::default(),
            timeout: None,
        })))
    }

    pub fn record_magic_url<S: Into<String>>(self, url: S) -> Self {
        self.record_magic(file::FileUrl::Raw(url.into()))
    }

    pub fn record_magic(self, url: file::FileUrl) -> Self {
        self.add_seg(SegmentSend::Record(box_new!(record::DataSend, {
            file: url,
            magic: Magic(1),
            cache: Cache::default(),
            proxy: Proxy::default(),
            timeout: None,
        })))
    }

    pub fn video_url<S: Into<String>>(self, url: S) -> Self {
        self.video(file::FileUrl::Raw(url.into()))
    }

    pub fn video(self, url: file::FileUrl) -> Self {
        self.add_seg(SegmentSend::Video(box_new!(video::DataSend, {
            file: url,
            cache: Cache::default(),
            proxy: Proxy::default(),
            timeout: None,
        })))
    }

    pub fn at<S: Into<String>>(self, qq: S) -> Self {
        self.add_seg(SegmentSend::At(box_new!(at::Data, { qq: qq.into() })))
            .text(" ")
    }

    pub fn rps(self) -> Self {
        self.add_seg(SegmentSend::Rps(rps::Data {}))
    }

    pub fn dice(self) -> Self {
        self.add_seg(SegmentSend::Dice(dice::Data {}))
    }

    pub fn shake(self) -> Self {
        self.add_seg(SegmentSend::Shake(shake::Data {}))
    }

    pub fn poke<S: Into<String>>(self, qq: S) -> Self {
        self.add_seg(SegmentSend::Poke(box_new!(poke::DataSend, {
            r#type: "1".to_string(),
            id: qq.into(),
        })))
    }

    pub fn anonymous(self) -> Self {
        self.add_seg(SegmentSend::Anonymous(anonymous::DataSend { ignore: None }))
    }

    pub fn share<S1: Into<String>, S2: Into<String>>(self, url: S1, title: S2) -> Self {
        self.add_seg(SegmentSend::Share(box_new!(share::DataSend, {
            url: url.into(),
            title: title.into(),
            content: None,
            image: None,
        })))
    }

    pub fn contact_friend<S: Into<String>>(self, qq: S) -> Self {
        self.add_seg(SegmentSend::Contact(box_new!(
            contact::Data,
            contact::Data::Qq(contact::Qq { id: qq.into() },)
        )))
    }

    pub fn contact_group<S: Into<String>>(self, group_id: S) -> Self {
        self.add_seg(SegmentSend::Contact(box_new!(
            contact::Data,
            contact::Data::Group(contact::Group {
                id: group_id.into(),
            },)
        )))
    }

    pub fn location(self, lat: f64, lon: f64) -> Self {
        self.add_seg(SegmentSend::Location(box_new!(location::DataSend, {
            lat: lat.to_string(),
            lon: lon.to_string(),
            title: None,
            content: None,
        })))
    }

    pub fn music_qq<S: Into<String>>(self, music_id: S) -> Self {
        self.add_seg(SegmentSend::Music(box_new!(
            music::Data,
            music::Data::Qq {
                id: music_id.into(),
            }
        )))
    }

    pub fn music_163<S: Into<String>>(self, music_id: S) -> Self {
        self.add_seg(SegmentSend::Music(box_new!(
            music::Data,
            music::Data::NetEase163 {
                id: music_id.into(),
            }
        )))
    }

    pub fn music_xiami<S: Into<String>>(self, music_id: S) -> Self {
        self.add_seg(SegmentSend::Music(box_new!(
            music::Data,
            music::Data::Xm {
                id: music_id.into(),
            }
        )))
    }

    pub fn music_custom<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        self,
        title: S1,
        share_url: S2,
        audio_url: S3,
    ) -> Self {
        self.add_seg(SegmentSend::Music(box_new!(
            music::Data,
            music::Data::Custom {
                title: title.into(),
                url: share_url.into(),
                audio: audio_url.into(),
                content: None,
                image: None,
            }
        )))
    }

    pub fn reply<S: Into<String>>(self, msg_id: S) -> Self {
        self.add_seg(SegmentSend::Reply(box_new!(reply::Data, {
            id: msg_id.into(),
        })))
    }

    pub fn node_id<S: Into<String>>(self, node_id: S) -> Self {
        self.add_seg(SegmentSend::Node(box_new!(
            node::DataSend,
            node::DataSend::Id(node::DataSend1 { id: node_id.into() },)
        )))
    }

    pub fn node_content<S1: Into<String>, S2: Into<String>>(
        self,
        user_id: S1,
        nickname: S2,
        content: MessageSend,
    ) -> Self {
        self.add_seg(SegmentSend::Node(box_new!(
            node::DataSend,
            node::DataSend::Content(node::DataSend2 {
                user_id: user_id.into(),
                nickname: nickname.into(),
                content: box_new!(MessageSend, content),
            },)
        )))
    }

    pub fn xml<S: Into<String>>(self, data: S) -> Self {
        self.add_seg(SegmentSend::Xml(box_new!(xml::Data, {
            data: data.into(),
        })))
    }

    pub fn json<S: Into<String>>(self, data: S) -> Self {
        self.add_seg(SegmentSend::Json(box_new!(json::Data, {
            data: data.into(),
        })))
    }
}

impl MessageSend {
    pub fn new_message() -> MessageSendBuilder {
        MessageSendBuilder::new()
    }
}
