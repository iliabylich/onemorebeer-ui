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
    let rendered = if cfg!(debug_assertions) {
        tokio::fs::read_to_string("templates/root.html")
            .await
            .context("failed to read template from disk")
    } else {
        RootTemplate {}
            .render()
            .context("failed to render static template")
    };

    Html(rendered.unwrap_or_else(|err| {
        log::error!("{err:?}");
        format!("internal server error")
    }))
}

async fn beers() -> impl IntoResponse {
    let json = Database::read()
        .await
        .and_then(|db| serde_json::to_value(db).context("failed to serialize DB to json"))
        .unwrap_or_else(|err| {
            log::error!("{err:?}");
            serde_json::json!({ "error": "internal server error" })
        });
    Json(json)
}
