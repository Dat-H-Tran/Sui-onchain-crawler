use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct SuiResponse {
    pub result: SuiResult,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SuiResult {
    Event {
        data: Vec<SuiEventData>,

        #[serde(rename = "nextCursor")]
        next_cursor: Value,
    },
    Package {
        status: String,
        details: SuiPackageDetails,
    },
}

#[derive(Deserialize, Debug)]
pub struct SuiPackageDetails {
    pub data: SuiPackageData,
}

#[derive(Deserialize, Debug)]
pub struct SuiPackageData {
    #[serde(rename = "dataType")]
    pub data_type: String,
    pub disassembled: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct SuiEventData {
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
