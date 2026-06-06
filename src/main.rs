use aws_sdk_route53::types::{
    Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType,
};
use env_logger::Env;
use log::{Level, debug, error};

const CHECKIP_URL: &'static str = "https://checkip.amazonaws.com/";

// Helper to easily map results into strings
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

    if let Err(e) = beacon("staticlinkage.dev".to_string()).await {
        error!("{e}");
    }
}

async fn beacon(host: String) -> Result<(), String> {
    let ip = get_ip().await.context("Failed to get IP")?;
    debug!("Resolved ip as: {ip}");

    update_dns(host, ip).await?;

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

async fn update_dns(host: String, ip: String) -> Result<(), String> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_route53::Client::new(&config);

    // client.get_hosted_zone().id(host.clone()).send().await.context("Unable to find hosted zone")?;

    // zone.hosted_zone().unwrap().

    let record = ResourceRecord::builder()
        .value(ip)
        .build()
        .context("Failed to build ResourceRecord")?;

    // TODO: support for subdomains
    let record_set = ResourceRecordSet::builder()
        .name(host.clone())
        .r#type(RrType::A)
        .ttl(300)
        .resource_records(record)
        .build()
        .context("Failed to build ResourceRecordSet")?;

    let change = Change::builder()
        .action(ChangeAction::Upsert)
        .resource_record_set(record_set)
        .build()
        .context("Failed to build change")?;

    let change_batch = ChangeBatch::builder()
        .changes(change)
        .build()
        .context("Failed to build ChangeBatch")?;

    client
        .change_resource_record_sets()
        .hosted_zone_id(host)
        .change_batch(change_batch)
        .send()
        .await
        .context("Failed to update DNS records")?;

    Ok(())
}
