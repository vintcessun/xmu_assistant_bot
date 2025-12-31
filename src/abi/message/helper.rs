use super::MessageSend;
use super::message_body::*;

pub struct MessageSendBuilder {
    segments: Vec<SegmentSend>,
}

impl MessageSendBuilder {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn build(self) -> MessageSend {
        MessageSend::Array(self.segments)
    }

    pub fn add(mut self, segment: SegmentSend) -> Self {
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

    pub fn text<S: Into<String>>(self, s: S) -> Self {
        self.add(SegmentSend::Text(text::Data { text: s.into() }))
    }

    pub fn face<S: Into<String>>(self, id: S) -> Self {
        self.add(SegmentSend::Face(face::Data { id: id.into() }))
    }

    pub fn image<S: Into<String>>(self, url: S) -> Self {
        self.add(SegmentSend::Image(image::DataSend {
            file: url.into(),
            cache: Cache::default(),
            proxy: Proxy::default(),
            timeout: None,
            r#type: None,
        }))
    }

    pub fn flash_image<S: Into<String>>(self, url: S) -> Self {
        self.add(SegmentSend::Image(image::DataSend {
            file: url.into(),
            cache: Cache::default(),
            proxy: Proxy::default(),
            timeout: None,
            r#type: Some("flash".to_string()),
        }))
    }

    pub fn record<S: Into<String>>(self, url: S) -> Self {
        self.add(SegmentSend::Record(record::DataSend {
            file: url.into(),
            magic: Magic::default(),
            cache: Cache::default(),
            proxy: Proxy::default(),
            timeout: None,
        }))
    }

    pub fn record_magic<S: Into<String>>(self, url: S) -> Self {
        self.add(SegmentSend::Record(record::DataSend {
            file: url.into(),
            magic: Magic(1),
            cache: Cache::default(),
            proxy: Proxy::default(),
            timeout: None,
        }))
    }

    pub fn video<S: Into<String>>(self, url: S) -> Self {
        self.add(SegmentSend::Video(video::DataSend {
            file: url.into(),
            cache: Cache::default(),
            proxy: Proxy::default(),
            timeout: None,
        }))
    }

    pub fn at<S: Into<String>>(self, qq: S) -> Self {
        self.add(SegmentSend::At(at::Data { qq: qq.into() }))
    }

    pub fn rps(self) -> Self {
        self.add(SegmentSend::Rps(rps::Data {}))
    }

    pub fn dice(self) -> Self {
        self.add(SegmentSend::Dice(dice::Data {}))
    }

    pub fn shake(self) -> Self {
        self.add(SegmentSend::Shake(shake::Data {}))
    }

    pub fn poke<S: Into<String>>(self, qq: S) -> Self {
        self.add(SegmentSend::Poke(poke::DataSend {
            r#type: "1".to_string(),
            id: qq.into(),
        }))
    }

    pub fn anonymous(self) -> Self {
        self.add(SegmentSend::Anonymous(anonymous::DataSend { ignore: None }))
    }

    pub fn share<S1: Into<String>, S2: Into<String>>(self, url: S1, title: S2) -> Self {
        self.add(SegmentSend::Share(share::DataSend {
            url: url.into(),
            title: title.into(),
            content: None,
            image: None,
        }))
    }

    pub fn contact_friend<S: Into<String>>(self, qq: S) -> Self {
        self.add(SegmentSend::Contact(contact::Data::Qq { id: qq.into() }))
    }

    pub fn contact_group<S: Into<String>>(self, group_id: S) -> Self {
        self.add(SegmentSend::Contact(contact::Data::Group {
            id: group_id.into(),
        }))
    }

    pub fn location(self, lat: f64, lon: f64) -> Self {
        self.add(SegmentSend::Location(location::DataSend {
            lat: lat.to_string(),
            lon: lon.to_string(),
            title: None,
            content: None,
        }))
    }

    pub fn music_qq<S: Into<String>>(self, music_id: S) -> Self {
        self.add(SegmentSend::Music(music::Data::Qq {
            id: music_id.into(),
        }))
    }

    pub fn music_163<S: Into<String>>(self, music_id: S) -> Self {
        self.add(SegmentSend::Music(music::Data::NetEase163 {
            id: music_id.into(),
        }))
    }

    pub fn music_xiami<S: Into<String>>(self, music_id: S) -> Self {
        self.add(SegmentSend::Music(music::Data::Xm {
            id: music_id.into(),
        }))
    }

    pub fn music_custom<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        self,
        title: S1,
        share_url: S2,
        audio_url: S3,
    ) -> Self {
        self.add(SegmentSend::Music(music::Data::Custom {
            title: title.into(),
            url: share_url.into(),
            audio: audio_url.into(),
            content: None,
            image: None,
        }))
    }

    pub fn reply<S: Into<String>>(self, msg_id: S) -> Self {
        self.add(SegmentSend::Reply(reply::Data { id: msg_id.into() }))
    }

    pub fn node_id<S: Into<String>>(self, node_id: S) -> Self {
        self.add(SegmentSend::Node(Box::new(node::DataSend::Id(
            node::DataSend1 { id: node_id.into() },
        ))))
    }

    pub fn node_content<S1: Into<String>, S2: Into<String>>(
        self,
        user_id: S1,
        nickname: S2,
        content: MessageSend,
    ) -> Self {
        self.add(SegmentSend::Node(Box::new(node::DataSend::Content(
            node::DataSend2 {
                user_id: user_id.into(),
                nickname: nickname.into(),
                content: Box::new(content),
            },
        ))))
    }

    pub fn xml<S: Into<String>>(self, data: S) -> Self {
        self.add(SegmentSend::Xml(xml::Data { data: data.into() }))
    }

    pub fn json<S: Into<String>>(self, data: S) -> Self {
        self.add(SegmentSend::Json(json::Data { data: data.into() }))
    }
}

impl MessageSend {
    pub fn new_message() -> MessageSendBuilder {
        MessageSendBuilder::new()
    }
}
