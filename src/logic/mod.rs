use crate::abi::logic_import::*;

mod echo;
mod login;

register_handlers!(echo::EchoHandler, login::LoginHandler, login::LogoutHandler);
