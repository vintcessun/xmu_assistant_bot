use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Target {
    Group(i64),
    Private(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Message,
    Notice,
    Request,
}

pub trait MessageType {
    fn get_target(&self) -> Target;
    fn get_type(&self) -> Type;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "post_type", rename_all = "snake_case")]
pub enum Event {
    Message(Box<message::Message>),
    Notice(notice::Notice),
    Request(request::Request),
    MetaEvent(meta::MetaEvent),
}

pub mod message {
    use crate::abi::message::{MessageReceive, SenderGroup, SenderPrivate};

    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "message_type", rename_all = "snake_case")]
    pub enum Message {
        Private(Private),
        Group(Group),
    }

    impl MessageType for Message {
        fn get_target(&self) -> Target {
            match self {
                Message::Private(private) => Target::Private(private.user_id),
                Message::Group(group) => Target::Group(group.group_id),
            }
        }

        fn get_type(&self) -> Type {
            Type::Message
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum SubTypePrivate {
        Friend,
        Group,
        Other,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Private {
        pub time: i64,
        pub self_id: i64,
        pub sub_type: SubTypePrivate,
        pub message_id: i32,
        pub user_id: i64,
        pub message: MessageReceive,
        pub raw_message: String,
        pub font: i32,
        pub sender: SenderPrivate,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum SubTypeGroup {
        Normal,
        Anonymous,
        Notice,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Anonymous {
        pub id: i64,
        pub name: String,
        pub flag: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Group {
        pub time: i64,
        pub self_id: i64,
        pub sub_type: SubTypeGroup,
        pub message_id: i32,
        pub group_id: i64,
        pub user_id: i64,
        pub anonymous: Option<Anonymous>,
        pub message: MessageReceive,
        pub raw_message: String,
        pub font: i32,
        pub sender: SenderGroup,
    }
}

pub mod notice {
    use crate::abi::message::file::File;

    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "notice_type", rename_all = "snake_case")]
    pub enum Notice {
        GroupUpload(GroupUpload),
        GroupAdmin(GroupAdmin),
        GroupDecrease(GroupDecrease),
        GroupIncrease(GroupIncrease),
        GroupBan(GroupBan),
        FriendAdd(FriendAdd),
        GroupRecall(GroupRecall),
        FriendRecall(FriendRecall),
        Notify(Notify),
    }

    impl MessageType for Notice {
        fn get_target(&self) -> Target {
            match self {
                Notice::GroupUpload(n) => Target::Group(n.group_id),
                Notice::GroupAdmin(n) => Target::Group(n.group_id),
                Notice::GroupDecrease(n) => Target::Group(n.group_id),
                Notice::GroupIncrease(n) => Target::Group(n.group_id),
                Notice::GroupBan(n) => Target::Group(n.group_id),
                Notice::FriendAdd(n) => Target::Private(n.user_id),
                Notice::GroupRecall(n) => Target::Group(n.group_id),
                Notice::FriendRecall(n) => Target::Private(n.user_id),
                Notice::Notify(notify) => match notify {
                    Notify::Poke(poke) => Target::Group(poke.group_id),
                    Notify::LuckyKing(lucky_king) => Target::Group(lucky_king.group_id),
                    Notify::Honor(honor) => Target::Group(honor.group_id),
                },
            }
        }

        fn get_type(&self) -> Type {
            Type::Notice
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GroupUpload {
        pub time: i64,
        pub self_id: i64,
        pub group_id: i64,
        pub user_id: i64,
        pub file: File,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum SubTypeGroupAdmin {
        Set,
        Unset,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GroupAdmin {
        pub time: i64,
        pub self_id: i64,
        pub sub_type: SubTypeGroupAdmin,
        pub group_id: i64,
        pub user_id: i64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum SubTypeGroupDecrease {
        Leave,
        Kick,
        KickMe,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GroupDecrease {
        pub time: i64,
        pub self_id: i64,
        pub sub_type: SubTypeGroupDecrease,
        pub group_id: i64,
        pub operator_id: i64,
        pub user_id: i64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum SubTypeGroupIncrease {
        Approve,
        Invite,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GroupIncrease {
        pub time: i64,
        pub self_id: i64,
        pub sub_type: SubTypeGroupIncrease,
        pub group_id: i64,
        pub operator_id: i64,
        pub user_id: i64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum SubTypeGroupBan {
        Ban,
        LiftBan,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GroupBan {
        pub time: i64,
        pub self_id: i64,
        pub sub_type: SubTypeGroupBan,
        pub group_id: i64,
        pub operator_id: i64,
        pub user_id: i64,
        pub duration: i64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct FriendAdd {
        pub time: i64,
        pub self_id: i64,
        pub user_id: i64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GroupRecall {
        pub time: i64,
        pub self_id: i64,
        pub group_id: i64,
        pub user_id: i64,
        pub operator_id: i64,
        pub message_id: i64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct FriendRecall {
        pub time: i64,
        pub self_id: i64,
        pub user_id: i64,
        pub message_id: i64,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum Notify {
        Poke(notify::Poke),
        LuckyKing(notify::LuckyKing),
        Honor(notify::Honor),
    }

    mod notify {
        use super::*;

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Poke {
            pub time: i64,
            pub self_id: i64,
            pub group_id: i64,
            pub user_id: i64,
            pub target_id: i64,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct LuckyKing {
            pub time: i64,
            pub self_id: i64,
            pub group_id: i64,
            pub user_id: i64,
            pub target_id: i64,
        }

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(rename_all = "snake_case")]
        pub enum HonorType {
            Talkative,
            Performer,
            Emotion,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Honor {
            pub time: i64,
            pub self_id: i64,
            pub group_id: i64,
            pub honor_type: HonorType,
            pub user_id: i64,
        }
    }
}

pub mod request {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "request_type", rename_all = "snake_case")]
    pub enum Request {
        Friend(Friend),
        Group(Group),
    }

    impl MessageType for Request {
        fn get_target(&self) -> Target {
            match self {
                Request::Friend(n) => Target::Private(n.user_id),
                Request::Group(n) => Target::Group(n.group_id),
            }
        }

        fn get_type(&self) -> Type {
            Type::Request
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Friend {
        pub time: i64,
        pub self_id: i64,
        pub user_id: i64,
        pub comment: String,
        pub flag: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum SubType {
        Add,
        Invite,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Group {
        pub time: i64,
        pub self_id: i64,
        pub sub_type: SubType,
        pub group_id: i64,
        pub user_id: i64,
        pub comment: String,
        pub flag: String,
    }
}

pub mod meta {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case", tag = "meta_event_type")]
    pub enum MetaEvent {
        Lifecycle(Lifecycle),
        Heartbeat(Heartbeat),
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum SubType {
        Enable,
        Disable,
        Connect,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Lifecycle {
        pub time: i64,
        pub self_id: i64,
        pub sub_type: SubType,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Status {
        online: bool,
        good: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Heartbeat {
        pub time: i64,
        pub self_id: i64,
        pub status: Status,
        pub interval: i64,
    }
}
