mod schedule;
mod zzy;

use std::sync::LazyLock;

pub use schedule::*;
pub use zzy::*;

use async_trait::async_trait;
use url::Url;
use url_macro::url;

pub static IDS_URL: LazyLock<Url> = LazyLock::new(|| url!("https://ids.xmu.edu.cn/authserver"));

#[async_trait]
pub trait JwAPI {
    const URL_DATA: &'static str;
    const APP_ENTRANCE: &'static str;
}
