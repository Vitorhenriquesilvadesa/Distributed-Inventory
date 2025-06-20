use common_models::ServiceInfo;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub registered_services: Arc<Mutex<HashMap<String, ServiceInfo>>>,
}
