mod beer;
mod config;
mod local_cache_middleware;
mod mode;
mod onemorebeer;
mod sync;
mod templates;
mod untappd;
mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    config::Config::load()?;

    match mode::parse_mode() {
        mode::Mode::Sync => {
            sync::Sync::run().await?;
        }
        mode::Mode::Web => {
            web::Web::spawn().await?;
        }
    }

    Ok(())
}
