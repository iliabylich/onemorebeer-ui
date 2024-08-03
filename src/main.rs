mod beer;
mod cache;
mod config;
mod mode;
mod onemorebeer;
mod ratebeer;
mod sync;
mod templates;
mod untappd;
mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    config::Config::load()?;

    match mode::parse_mode()? {
        mode::Mode::Sync => {
            sync::Sync::run().await?;
        }
        mode::Mode::Web => {
            web::Web::spawn().await?;
        }
    }

    Ok(())
}
