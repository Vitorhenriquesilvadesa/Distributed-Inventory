use super::state::AppState;
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use common_models::{ServiceInfo, ServiceInfoLookup};
use std::time::Duration;
use tokio::time::sleep;

/// Registra um novo serviço ou atualiza um já existente.
#[utoipa::path(
    post,
    path = "/register",
    request_body = ServiceInfo,
    responses(
        (status = 200, description = "Service registered successfully", body = String, example = json!("Service cd_alpha registered successfully")),
    ),
    tag = "Service Discovery"
)]
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

/// Busca os detalhes de conexão de um serviço específico.
#[utoipa::path(
    get,
    path = "/lookup/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the service to lookup")
    ),
    responses(
        (status = 200, description = "Service details found", body = ServiceInfoLookup),
        (status = 404, description = "Service not found", body = String, example = json!("Service cd_omega not found"))
    ),
    tag = "Service Discovery"
)]
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

/// Lista os IDs de todos os serviços registrados e ativos.
#[utoipa::path(
    get,
    path = "/lookup_all",
    responses(
        (status = 200, description = "List of all registered service IDs", body = Vec<String>, example = json!(["cd_alpha", "cd_beta"])),
    ),
    tag = "Service Discovery"
)]
pub async fn lookup_all_services(data: web::Data<AppState>) -> impl Responder {
    let services = data.registered_services.lock().unwrap();
    let service_ids: Vec<String> = services.keys().cloned().collect();
    HttpResponse::Ok().json(service_ids)
}

/// Recebe um sinal de "heartbeat" de um serviço.
#[utoipa::path(
    post,
    path = "/heartbeat/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the service sending the heartbeat")
    ),
    responses(
        (status = 200, description = "Heartbeat received", body = String, example = json!("Heartbeat received for cd_alpha")),
        (status = 404, description = "Service not found", body = String, example = json!("Service cd_omega not found for heartbeat"))
    ),
    tag = "Service Discovery"
)]
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

/// Executa um ciclo de limpeza em segundo plano para remover serviços inativos.
///
/// Esta não é um endpoint de API, mas uma tarefa assíncrona que roda continuamente.
/// A cada 10 segundos, ela verifica todos os serviços registrados e remove aqueles
/// cujo último heartbeat foi há mais de 30 segundos.
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