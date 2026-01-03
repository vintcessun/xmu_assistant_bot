mod profile;

use std::sync::LazyLock;

pub use profile::*;

use url::Url;
use url_macro::url;

pub static LNT_URL: LazyLock<Url> = LazyLock::new(|| url!("https://lnt.xmu.edu.cn"));
