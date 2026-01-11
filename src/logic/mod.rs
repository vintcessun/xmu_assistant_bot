use crate::register_handler_with_help;

mod download;
mod echo;
mod helper;
mod login;

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
    other = []
);
