use crate::abi::logic_import::*;

mod download;
mod echo;
mod helper;
mod login;
mod search;

register_handlers!(
    echo::EchoHandler,
    login::LoginHandler,
    login::LogoutHandler,
    download::DownloadHandler
);
