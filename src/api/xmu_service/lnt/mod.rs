mod profile;

pub use profile::*;

use once_cell::sync::Lazy;
use url::Url;
use url_macro::url;

pub static LNT_URL: Lazy<Url> = Lazy::new(|| url!("https://lnt.xmu.edu.cn"));
