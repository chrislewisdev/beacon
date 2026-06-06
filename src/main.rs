mod context;

use aws_sdk_route53::types::{
    Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType,
};
use clap::Parser;
use context::{AwsErrorContext, ErrorContext};
use env_logger::Env;
use log::{Level, debug, error, info};
use tokio::time::{MissedTickBehavior, interval};
use std::{process::exit, time::Duration};

const CHECKIP_URL: &'static str = "https://checkip.amazonaws.com/";

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(long)]
    zone_name: String,
    #[arg(long)]
    subdomain: Vec<String>,
    #[arg(long)]
    update_root: bool,
    #[arg(long, default_value_t = 0)]
    interval: u64,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(Level::Info.to_string())).init();

    let args = CliArgs::parse();
    let domains = get_domains_to_update(&args);

    if domains.len() == 0 {
        error!(
            "No domains were specified to update. Either supply at least one --subdomain or specify --update-root to update the root record."
        );
        exit(1);
    }

    let execute = async || {
        if let Err(e) = beacon(&args.zone_name, &domains).await {
            error!("{e}");
            error!("Failed to update DNS. See error logs for details.");
            false
        } else {
            true
        }
    };

    if args.interval == 0 {
        if !execute().await {
            exit(1);
        }
    } else {
        let mut interval = interval(Duration::from_secs(args.interval));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            execute().await;
        }
    }
}

fn get_domains_to_update(args: &CliArgs) -> Vec<String> {
    let mut domains: Vec<String> = args
        .subdomain
        .iter()
        .map(|subdomain| format!("{}.{}", subdomain, args.zone_name))
        .collect();

    if args.update_root {
        domains.push(args.zone_name.clone());
    }

    domains
}

async fn beacon(zone_name: &String, domains: &Vec<String>) -> Result<(), String> {
    let ip = get_ip().await.context("Failed to get IP")?;
    debug!("Resolved ip as: {ip}");

    update_dns(zone_name, domains, ip).await?;

    info!("DNS for {} was successfully updated.", zone_name);

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

async fn update_dns(zone_name: &String, domains: &Vec<String>, ip: String) -> Result<(), String> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_route53::Client::new(&config);

    let zone_id = get_zone_id(&client, &zone_name).await?;

    let change_batch = build_dns_change_batch(domains, ip)?;

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
        .ok_or(format!("No hosted zone found matching '{}'", zone_name))?
        .id();

    Ok(zone_id.to_string())
}

fn build_dns_change_batch(domains: &Vec<String>, ip: String) -> Result<ChangeBatch, String> {
    let ip_record = ResourceRecord::builder()
        .value(ip)
        .build()
        .context("Failed to build ResourceRecord")?;

    let mut changes: Vec<Change> = Vec::new();
    for domain in domains {
        let record_set = ResourceRecordSet::builder()
            .name(domain)
            .r#type(RrType::A)
            .ttl(300)
            .resource_records(ip_record.clone())
            .build()
            .context("Failed to build ResourceRecordSet")?;

        let change = Change::builder()
            .action(ChangeAction::Upsert)
            .resource_record_set(record_set)
            .build()
            .context("Failed to build Change")?;

        changes.push(change);
    }

    let change_batch = ChangeBatch::builder()
        .set_changes(Some(changes))
        .build()
        .context("Failed to build ChangeBatch")?;

    Ok(change_batch)
}
