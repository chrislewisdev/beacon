use std::process::exit;

use aws_sdk_route53::{
    error::{ProvideErrorMetadata, SdkError},
    types::{Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType},
};
use env_logger::Env;
use log::{Level, debug, error, info};

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

trait AwsContext<R> {
    fn aws_context(self, c: &str) -> Result<R, String>;
}
impl<R, E> AwsContext<R> for Result<R, SdkError<E>>
where
    E: ProvideErrorMetadata,
{
    fn aws_context(self, c: &str) -> Result<R, String> {
        self.map_err(|e| format!("{}: {}", c, e.message().unwrap_or("Unknown sdk error")))
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(Level::Info.to_string())).init();

    if let Err(e) = beacon("staticlinkage.dev".to_string()).await {
        error!("{e}");
        error!("Failed to update DNS. See error logs for details.");
        exit(1);
    } else {
        info!("DNS was successfully updated.");
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

    let zone_id = get_zone_id(&client, &host).await?;

    let change_batch = build_dns_change_batch(host, ip)?;

    client
        .change_resource_record_sets()
        .hosted_zone_id(zone_id)
        .change_batch(change_batch)
        .send()
        .await
        .aws_context("Failed to update Route53")?;

    Ok(())
}

async fn get_zone_id(client: &aws_sdk_route53::Client, host: &String) -> Result<String, String> {
    let zones = client
        .list_hosted_zones()
        .send()
        .await
        .aws_context("Failed to list hosted zones")?;

    // Route53 includes a trailing '.' on all zone names.
    let zone_name = if host.ends_with(".") {
        host.clone()
    } else {
        host.clone() + "."
    };

    let zone_id = zones
        .hosted_zones()
        .iter()
        .find(|z| z.name() == zone_name)
        .ok_or(format!("No hosted zone found matching {}.", zone_name))?
        .id();

    Ok(zone_id.to_string())
}

fn build_dns_change_batch(host: String, ip: String) -> Result<ChangeBatch, String> {
    let record = ResourceRecord::builder()
        .value(ip)
        .build()
        .context("Failed to build ResourceRecord")?;

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

    Ok(change_batch)
}
