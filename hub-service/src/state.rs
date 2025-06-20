use common_models::Product;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub products_catalog: Arc<Mutex<HashMap<String, Product>>>,
    pub service_discovery_url: String,
    pub http_client: Client,
}
