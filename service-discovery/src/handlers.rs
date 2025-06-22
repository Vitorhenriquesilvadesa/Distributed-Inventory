use super::state::AppState;
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use common_models::{ServiceInfo, ServiceInfoLookup};
use std::time::Duration;
use tokio::time::sleep;

/// **POST /register**
///
/// Registra um novo serviço ou atualiza um já existente no sistema.
///
/// Este endpoint é a porta de entrada para qualquer serviço (como um `CD Service`)
/// se anunciar na rede. Ele recebe os detalhes do serviço, define o `last_heartbeat`
/// para o momento atual e o insere no registro central.
///
/// # Corpo da Requisição (JSON)
///
/// ```json
/// {
///   "id": "cd_alpha",
///   "ip": "127.0.0.1",
///   "port": 8083
/// }
/// ```
///
/// # Resposta de Sucesso (200 OK)
///
/// Retorna uma mensagem de texto confirmando o registro.
///
/// ```text
/// Service cd_alpha registered successfully
/// ```
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

/// **GET /lookup/{id}**
///
/// Busca e retorna os detalhes de conexão de um serviço específico pelo seu ID.
///
/// Este endpoint permite que um serviço encontre o endereço de outro para comunicação
/// direta. Retorna uma struct `ServiceInfoLookup` contendo os dados para a conexão.
///
/// # Parâmetros de Rota
///
/// * `id`: O identificador único do serviço a ser buscado (ex: "cd_alpha").
///
/// # Resposta de Sucesso (200 OK)
///
/// Retorna um objeto JSON com as informações do serviço.
///
/// ```json
/// {
///   "id": "cd_alpha",
///   "ip": "127.0.0.1",
///   "port": 8083
/// }
/// ```
///
/// # Resposta de Erro (404 Not Found)
///
/// Retorna uma mensagem de texto se o serviço não for encontrado.
///
/// ```text
/// Service cd_alpha not found
/// ```
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

/// **GET /lookup_all**
///
/// Lista os IDs de todos os serviços atualmente registrados e ativos.
///
/// Este endpoint é usado pelo `Hub Service` para obter uma lista de todos os CDs
/// que ele precisa consultar.
///
/// # Resposta de Sucesso (200 OK)
///
/// Retorna um array JSON contendo os IDs (strings) de todos os serviços registrados.
///
/// ```json
/// [
///   "cd_gamma",
///   "cd_alpha",
///   "cd_beta"
/// ]
/// ```
pub async fn lookup_all_services(data: web::Data<AppState>) -> impl Responder {
    let services = data.registered_services.lock().unwrap();
    let service_ids: Vec<String> = services.keys().cloned().collect();
    HttpResponse::Ok().json(service_ids)
}

/// **POST /heartbeat/{id}**
///
/// Recebe um sinal de "heartbeat" de um serviço, atualizando seu estado para "ativo".
///
/// Os serviços devem chamar este endpoint periodicamente para provar que ainda estão online.
/// A função simplesmente atualiza o timestamp `last_heartbeat` do serviço correspondente.
///
/// # Parâmetros de Rota
///
/// * `id`: O identificador único do serviço enviando o heartbeat (ex: "cd_alpha").
///
/// # Resposta de Sucesso (200 OK)
///
/// ```text
/// Heartbeat received for cd_alpha
/// ```
///
/// # Resposta de Erro (404 Not Found)
///
/// ```text
/// Service cd_alpha not found for heartbeat
/// ```
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
