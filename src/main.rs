use env_logger::Env;
use log::{Level, debug, error};

const CHECKIP_URL: &'static str = "https://checkip.amazonaws.com/";

trait ErrorContext<R> {
    fn context(self, c: &str) -> Result<R, String>;
}

impl<R, E> ErrorContext<R> for Result<R, E>
where
    E: ToString,
{
    fn context(self, c: &str) -> Result<R, String> {
        self.map_err(|e| format!("{}: {}", c, e.to_string()))
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(Level::Info.to_string())).init();

    if let Err(e) = beacon().await {
        error!("{e}");
    }
}

async fn beacon() -> Result<(), String> {
    let ip = get_ip().await.context("Failed to get IP")?;
    debug!("Resolved ip as: {ip}");

    Ok(())
}

async fn get_ip() -> reqwest::Result<String> {
    Ok(reqwest::get(CHECKIP_URL)
        .await?
        .text()
        .await?
        .trim()
        .to_string())
}
