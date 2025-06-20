use super::state::AppState;
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use common_models::{ServiceInfo, ServiceInfoLookup};
use std::time::Duration;
use tokio::time::sleep;

pub async fn register_service(
    info: web::Json<ServiceInfo>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut services = data.registered_services.lock().unwrap();
    let mut service_info = info.clone();
    service_info.last_heartbeat = Utc::now();
    services.insert(service_info.id.clone(), service_info);
    println!("Registered/Updated service: {:?}", services.get(&info.id));
    HttpResponse::Ok().body(format!("Service {} registered successfully", info.id))
}

pub async fn lookup_service(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let service_id = path.into_inner();
    let services = data.registered_services.lock().unwrap();

    if let Some(info) = services.get(&service_id) {
        HttpResponse::Ok().json(ServiceInfoLookup {
            id: info.id.clone(),
            ip: info.ip.clone(),
            port: info.port,
        })
    } else {
        HttpResponse::NotFound().body(format!("Service {} not found", service_id))
    }
}

pub async fn lookup_all_services(data: web::Data<AppState>) -> impl Responder {
    let services = data.registered_services.lock().unwrap();
    let service_ids: Vec<String> = services.keys().cloned().collect();
    HttpResponse::Ok().json(service_ids)
}

pub async fn heartbeat(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let service_id = path.into_inner();
    let mut services = data.registered_services.lock().unwrap();

    if let Some(service_info) = services.get_mut(&service_id) {
        service_info.last_heartbeat = Utc::now();
        HttpResponse::Ok().body(format!("Heartbeat received for {}", service_id))
    } else {
        HttpResponse::NotFound().body(format!("Service {} not found for heartbeat", service_id))
    }
}

pub async fn cleanup_inactive_services(state: web::Data<AppState>) {
    let cleanup_interval = Duration::from_secs(10);
    let inactivity_threshold = Duration::from_secs(30);

    loop {
        sleep(cleanup_interval).await;
        let mut services = state.registered_services.lock().unwrap();
        let now = Utc::now();
        let mut removed_ids = Vec::new();

        services.retain(|id, info| {
            if (now - info.last_heartbeat).to_std().unwrap_or_default() > inactivity_threshold {
                println!("Removing inactive service: {}", id);
                removed_ids.push(id.clone());
                false
            } else {
                true
            }
        });

        if !removed_ids.is_empty() {
            println!("Cleaned up: {:?}", removed_ids);
        }
    }
}
