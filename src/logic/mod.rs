mod download;
mod echo;
mod helper;
mod llm;
mod login;

use crate::abi::logic_import::*;

pub trait BuildHelp {
    const HELP_MSG: &'static str;
}

register_handler_with_help!(
    command = [
        echo::EchoHandler,
        login::LoginHandler,
        login::LogoutHandler,
        download::DownloadHandler,
    ],
    other = [llm::LlmMessageHandler, llm::LlmNoticeHandler,]
);
