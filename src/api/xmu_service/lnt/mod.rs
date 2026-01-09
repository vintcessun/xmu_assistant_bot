mod activities;
mod file_url;
mod my_courses;
mod profile;
mod recently_visited_courses;

use std::sync::LazyLock;
use url::Url;
use url_macro::url;

pub use activities::Activities;
pub use file_url::FileUrl;
pub use my_courses::MyCourses;
pub use profile::Profile;
pub use recently_visited_courses::RecentlyVisitedCourses;

use crate::api::network::SessionClient;

pub static LNT_URL: LazyLock<Url> = LazyLock::new(|| url!("https://lnt.xmu.edu.cn"));

pub fn get_session_client(session: &str) -> SessionClient {
    let client = SessionClient::new();
    client.set_cookie("session", session, LNT_URL.clone());
    client
}
