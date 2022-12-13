use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct SuiResponse {
    pub result: SuiResult,
}

#[derive(Deserialize, Debug)]
pub struct SuiResult {
    pub data: Vec<SuiData>,
    
    #[serde(rename = "nextCursor")]
    pub next_cursor: Value,
}

#[derive(Deserialize, Debug)]
pub struct SuiData {
    #[serde(rename = "txDigest")]
    pub tx_digest: String,
    
    pub timestamp: i64,
    
    pub event: SuiPublishEvent,
}

#[derive(Deserialize, Debug)]
pub struct SuiPublishEvent {
    pub publish: SuiPublishDetail,
}

#[derive(Deserialize, Debug)]
pub struct SuiPublishDetail {
    pub sender: String,
    
    #[serde(rename = "packageId")]
    pub package_id: String,
}
