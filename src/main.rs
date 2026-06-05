use env_logger::Env;
use log::info;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Hello, world!");
}
