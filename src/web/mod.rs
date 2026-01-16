use anyhow::Result;
use axum::{Router, routing::get};

pub mod file;

use file::file_router;

const URL: &str = "https://zzy.vintces.icu";
const LOCAL: &str = "0.0.0.0:3080";

pub async fn start() -> Result<()> {
    let app = router();

    let listener = tokio::net::TcpListener::bind(LOCAL).await?;

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    Ok(())
}

fn router() -> Router {
    let router = Router::new();
    let router = router.nest("/file", file_router());
    main_router(router)
}

fn main_router(router: Router) -> Router {
    router.route("/status", get(status_handler))
}

async fn status_handler() -> &'static str {
    "Web API is running"
}
