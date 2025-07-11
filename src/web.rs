use crate::{beer::Database, config::Config, templates::RootTemplate};
use anyhow::{Context, Result};
use askama::Template;
use axum::{
    Json, Router,
    response::{Html, IntoResponse},
    routing::get,
};
use tokio::net::TcpListener;

pub(crate) struct Web;

impl Web {
    pub(crate) async fn spawn() -> Result<()> {
        let app = Router::new()
            .route("/", get(root))
            .route("/beers.json", get(beers));

        let port = Config::global().listen_on;
        let listener = TcpListener::bind(("127.0.0.1", port))
            .await
            .context("failed to bind")?;
        println!(
            "Listening on {}",
            listener.local_addr().context("failed to get local addr")?
        );

        axum::serve(listener, app)
            .await
            .context("Failed to spawn web server")
    }
}

async fn root() -> impl IntoResponse {
    if cfg!(debug_assertions) {
        Html(
            tokio::fs::read_to_string("templates/root.html")
                .await
                .unwrap(),
        )
    } else {
        Html(RootTemplate {}.render().unwrap())
    }
}

async fn beers() -> impl IntoResponse {
    match Database::read().await {
        Ok(Some(db)) => Json(serde_json::to_value(db).unwrap()),
        Ok(None) => Json(serde_json::json!({ "error": "No data" })),
        Err(err) => Json(serde_json::json!({ "error": format!("{:?}", err) })),
    }
}
