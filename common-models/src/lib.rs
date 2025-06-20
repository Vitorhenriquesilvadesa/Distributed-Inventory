use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub code: String,
    pub name: String,
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceInfo {
    pub id: String,
    pub ip: String,
    pub port: u16,
    #[serde(default = "Utc::now")]
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceInfoLookup {
    pub id: String,
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductAvailability {
    pub cd_id: String,
    pub quantity_available: u32,
    pub product_info: Product,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferRequest {
    pub product_code: String,
    pub quantity: u32,
    pub requester_cd_id: String,
}
