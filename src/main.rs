use std::env;

use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;

const SUI_DEVNET_FULLNODE: &str = "https://fullnode.devnet.sui.io:443";
const ENTRY_PER_PAGE: usize = 3;
const MAX_PAGE: usize = 10;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv()?;
    let database_url = env::var("DATABASE_URL")?;

    // Create a connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL)
    let _row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool)
        .await?;

    let client = reqwest::Client::new().post(SUI_DEVNET_FULLNODE);
    let mut next_page = Value::Null;
    
    for _ in 0..MAX_PAGE {
        let response = client
            .try_clone()
            .unwrap()
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "sui_getEvents",
                "params": [
                    {
                        "EventType": "Publish"
                    },
                    next_page,
                    ENTRY_PER_PAGE,
                    true
                ]
            }))
            .send()
            .await?;
        let response_json = response.json::<Value>().await?;
        let result = &response_json["result"];
        let data = result["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Response does not contain `result`."))?;
        let next = result["nextCursor"].to_string();

        for object in data.iter() {
            let sender = object["event"]["publish"]["sender"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Response does not contain `sender`."))?;
            let package_id = object["event"]["publish"]["packageId"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Response does not contain `packageId`."))?;
            let timestamp = object["timestamp"]
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("Response does not contain `timestamp`."))?;
            println!("{sender} published {package_id} at {timestamp}");
        }
        println!("{next}");

        next_page = result["nextCursor"].clone();
        if next_page.is_null() {
            break;
        }
    }

    Ok(())
}
