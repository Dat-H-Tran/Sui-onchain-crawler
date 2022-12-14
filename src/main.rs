mod prelude;

use crate::prelude::*;
use std::env;

use anyhow::bail;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

const SUI_DEVNET_FULLNODE: &str = "https://fullnode.devnet.sui.io:443";
const SUI_RELEASES_API: &str = "https://api.github.com/repos/MystenLabs/sui/releases";
const ENTRIES_PER_PAGE: usize = 1000;
const SORT_BY_LATEST: bool = true;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv()?;
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::migrate!("db/migrations").run(&pool).await?;

    let last_timestamp = get_last_timestamp(&pool).await?;

    let network_version = get_network_version().await?;

    collect_package_ids(last_timestamp, &network_version, &pool).await?;

    collect_package_contents(&pool).await?;

    Ok(())
}

async fn collect_package_ids(
    last_timestamp: Option<i64>,
    network_version: &str,
    pool: &Pool<Postgres>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new().post(SUI_DEVNET_FULLNODE);
    let mut next_page = Value::Null;
    let mut crawling = true;

    while crawling {
        let response = client
            .try_clone()
            .unwrap()
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "sui_getEvents",
                "params": [
                    { "EventType": "Publish" },
                    next_page,
                    ENTRIES_PER_PAGE,
                    SORT_BY_LATEST
                ]
            }))
            .send()
            .await?;
        let response = response.json::<SuiResponse>().await?;
        let SuiResult::Event { data, next_cursor } = response.result else {
            bail!("Not an event")
        };

        let mut num_of_ids = 0;
        for object in data.iter() {
            let sender = &object.event.publish.sender;
            let package_id = &object.event.publish.package_id;
            let tx_digest = &object.tx_digest;
            let timestamp = &object.timestamp;

            if let Some(last_ts) = last_timestamp {
                if timestamp <= &last_ts {
                    crawling = false;
                    break;
                }
            }

            let _row = sqlx::query!(
                "INSERT INTO sui_packages(package_id, sender, tx_digest, timestamp, network_version)\
                 VALUES ($1, $2, $3, $4, $5)",
                package_id,
                sender,
                tx_digest,
                timestamp,
                network_version
            )
            .execute(pool)
            .await?;

            num_of_ids += 1;
        }
        println!("Collected {num_of_ids} package ids...");

        next_page = next_cursor.clone();
        if next_page.is_null() {
            break;
        }
    }

    Ok(())
}

async fn get_network_version() -> Result<String, anyhow::Error> {
    let github_api_token = env::var("GITHUB_API_TOKEN")?;
    let response = reqwest::Client::new()
        .get(SUI_RELEASES_API)
        .header("User-Agent", "")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Authorization", format!("Bearer {github_api_token}"))
        .send()
        .await?;

    let response_json = response.json::<Vec<Value>>().await?;
    let version = response_json[0]["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing tag_name"))?;

    Ok(version.to_owned())
}

async fn get_last_timestamp(pool: &Pool<Postgres>) -> Result<Option<i64>, anyhow::Error> {
    let last_timestamp = sqlx::query!("SELECT max(timestamp) FROM sui_packages;")
        .fetch_one(pool)
        .await?;
    Ok(last_timestamp.max)
}

async fn collect_package_contents(pool: &Pool<Postgres>) -> Result<(), anyhow::Error> {
    // Paginated-ly load empty modules
    let batch = sqlx::query!("SELECT package_id FROM sui_packages WHERE content IS NULL")
        .fetch_all(pool)
        .await?;
    let batch = batch
        .iter()
        .map(|record| record.package_id.as_ref().unwrap())
        .collect::<Vec<_>>();

    // Get the contents of the package from sui
    let client = reqwest::Client::new().post(SUI_DEVNET_FULLNODE);

    for (index, package_id) in batch.iter().enumerate() {
        let package_id = *package_id;

        let response = client
            .try_clone()
            .unwrap()
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "sui_getObject",
                "params": [
                    package_id
                ]
            }))
            .send()
            .await?;

        let response = response.json::<SuiResponse>().await?;
        let SuiResult::Package { details, .. } = response.result else {
            bail!("Not a package")
        };
        let map = details.data.disassembled;

        let _affected_rows = sqlx::query!(
            "UPDATE sui_packages SET content = $1 WHERE package_id = $2",
            format!("{map:?}"),
            package_id
        )
        .execute(pool)
        .await?;

        if index % 50 == 0 {
            println!("Saved {} packages...", index + 1);
        }
    }
    Ok(())
}
