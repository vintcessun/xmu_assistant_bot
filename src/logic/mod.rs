use crate::abi::logic_import::*;
use helper::register_handlers;

mod echo;

register_handlers!(echo::EchoHandler);
