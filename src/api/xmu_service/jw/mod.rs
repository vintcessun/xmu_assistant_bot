mod schedule;
mod zzy;

pub use schedule::*;
pub use zzy::*;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use url::Url;
use url_macro::url;

pub static IDS_URL: Lazy<Url> = Lazy::new(|| url!("https://ids.xmu.edu.cn/authserver"));

#[async_trait]
pub trait JwAPI {
    const URL_DATA: &'static str;
    const APP_ENTRANCE: &'static str;
}
