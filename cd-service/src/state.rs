use common_models::Product;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct CdState {
    pub inventory: Arc<Mutex<HashMap<String, Product>>>,
    pub service_discovery_url: String,
    pub hub_url: String,
    pub http_client: Client,
    pub own_id: String,
}
