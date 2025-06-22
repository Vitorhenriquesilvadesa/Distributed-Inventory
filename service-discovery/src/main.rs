use actix_web::{web, App, HttpServer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod handlers;
mod state;

/// Ponto de entrada principal para o serviço de `Service Discovery`.
///
/// Esta função é responsável por configurar e executar o servidor central que gerencia
/// o registro de todos os outros serviços no sistema. Sua inicialização consiste em:
///
/// 1.  **Criação do Estado:** Inicializa o `AppState`, que contém o `HashMap` protegido
///     por um `Mutex` para armazenar os dados de todos os serviços registrados de forma
///     segura entre as threads.
/// 2.  **Inicialização da Tarefa de Limpeza:** Dispara a tarefa de fundo assíncrona
///     `cleanup_inactive_services`. Esta tarefa é crucial para a tolerância a falhas,
///     removendo serviços que não enviam heartbeats.
/// 3.  **Configuração do Servidor HTTP:** Inicia o servidor `actix-web`, registrando
///     todos os endpoints da API (`/register`, `/lookup`, `/heartbeat`, etc.) e
///     disponibilizando o estado compartilhado para os handlers.
///
/// # Returns
///
/// * `std::io::Result<()>` - Retorna `Ok(())` se o servidor for encerrado graciosamente,
///   ou um `Err` se houver uma falha ao vincular o servidor ao IP e porta (ex: porta já em uso).
///
/// # Side Effects
///
/// * Inicia uma tarefa assíncrona (`tokio::task`) que roda em um loop infinito em
///   segundo plano para limpar serviços inativos.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Cria o estado da aplicação, envolvendo o registro de serviços em um Arc<Mutex>
    // para permitir acesso compartilhado e seguro entre as threads do Actix.
    let app_state = web::Data::new(state::AppState {
        registered_services: Arc::new(Mutex::new(HashMap::new())),
    });

    // Clona o estado para movê-lo para a tarefa de limpeza em segundo plano.
    let cleanup_state = app_state.clone();
    // Dispara a tarefa que remove serviços inativos de forma concorrente.
    tokio::spawn(handlers::cleanup_inactive_services(cleanup_state));

    let ip = "127.0.0.1";
    let port = 8080;

    println!("Service Discovery running on http://{}:{}", ip, port);

    // Configura e inicia o servidor HTTP.
    HttpServer::new(move || {
        App::new()
            // Disponibiliza o estado para todos os handlers.
            .app_data(app_state.clone())
            // Registra os endpoints da API.
            .service(web::resource("/register").post(handlers::register_service))
            .service(web::resource("/lookup/{id}").get(handlers::lookup_service))
            .service(web::resource("/lookup_all").get(handlers::lookup_all_services))
            .service(web::resource("/heartbeat/{id}").post(handlers::heartbeat))
    })
    .bind(format!("{}:{}", ip, port))?
    .run()
    .await
}
