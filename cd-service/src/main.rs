use actix_web::{web, App, HttpServer};
use common_models::Product;
use common_models::ServiceInfo;
use reqwest::Client;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

mod client;
mod handlers;
mod state;

/// Ponto de entrada para a aplicação `CD Service`.
///
/// Esta função é responsável por inicializar, configurar e executar uma instância
/// de um Centro de Distribuição. O processo de inicialização inclui:
///
/// 1.  **Análise de Argumentos:** Lê o ID e a PORTA do CD a partir dos argumentos da linha de comando.
/// 2.  **Criação de Inventário Inicial:** Configura um inventário de produtos hardcoded
///     com base no ID do CD, para fins de demonstração.
/// 3.  **Inicialização do Estado:** Cria o estado compartilhado da aplicação (`CdState`),
///     que será acessível por todos os handlers da API.
/// 4.  **Registro no Service Discovery:** Envia uma requisição `POST /register` para o
///     `Service Discovery`, anunciando sua presença na rede. O processo é abortado
///     se o registro falhar.
/// 5.  **Inicialização de Tarefas de Fundo:**
///     - Dispara a tarefa `send_heartbeat` para enviar sinais de vida contínuos.
///     - Dispara tarefas de demonstração que solicitam produtos automaticamente para
///       simular a necessidade de reabastecimento.
/// 6.  **Inicialização do Servidor HTTP:** Configura e inicia o servidor `actix-web` com
///     os endpoints da API do CD (`/inventory`, `/transfer_product`, etc.).
///
/// # Returns
///
/// * `std::io::Result<()>` - Retorna `Ok(())` se o servidor foi encerrado graciosamente
///   ou um `Err` se o servidor não pôde ser vinculado ao IP e porta especificados.
///
/// # Panics
///
/// * A função causará um pânico (panic) se o segundo argumento da linha de comando (a porta)
///   não puder ser convertido para um número (`u16`), devido à chamada `.expect()`.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Analisa os argumentos da linha de comando para obter o ID e a porta do CD.
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <CD_ID> <PORT>", args[0]);
        std::process::exit(1);
    }

    let cd_id = args[1].clone();
    let port: u16 = args[2].parse().expect("Port must be a valid number");
    let ip = "127.0.0.1".to_string();

    let service_discovery_url = "http://127.0.0.1:8080".to_string();
    let hub_url = "http://127.0.0.1:8082".to_string();

    // 2. Define um inventário inicial diferente para cada CD, para fins de demonstração.
    let initial_inventory = {
        let mut map = HashMap::new();
        match cd_id.as_str() {
            "cd_alpha" => {
                map.insert(
                    "garrafas".to_string(),
                    Product {
                        code: "garrafas".to_string(),
                        name: "Garrafas de Água".to_string(),
                        price: 2.50,
                        quantity: Some(50),
                    },
                );
                map.insert(
                    "celulares".to_string(),
                    Product {
                        code: "celulares".to_string(),
                        name: "Smartphones X".to_string(),
                        price: 1200.00,
                        quantity: Some(10),
                    },
                );
            }
            "cd_beta" => {
                map.insert(
                    "garrafas".to_string(),
                    Product {
                        code: "garrafas".to_string(),
                        name: "Garrafas de Água".to_string(),
                        price: 2.50,
                        quantity: Some(30),
                    },
                );
                map.insert(
                    "cadernos".to_string(),
                    Product {
                        code: "cadernos".to_string(),
                        name: "Cadernos Espirais".to_string(),
                        price: 8.00,
                        quantity: Some(100),
                    },
                );
            }
            "cd_gamma" => {
                map.insert(
                    "celulares".to_string(),
                    Product {
                        code: "celulares".to_string(),
                        name: "Smartphones X".to_string(),
                        price: 1200.00,
                        quantity: Some(15),
                    },
                );
                map.insert(
                    "canetas".to_string(),
                    Product {
                        code: "canetas".to_string(),
                        name: "Canetas Esferográficas".to_string(),
                        price: 1.50,
                        quantity: Some(200),
                    },
                );
            }
            _ => {
                map.insert(
                    "generico".to_string(),
                    Product {
                        code: "generico".to_string(),
                        name: "Produto Genérico".to_string(),
                        price: 5.00,
                        quantity: Some(5),
                    },
                );
            }
        }
        map
    };

    // 3. Cria o estado compartilhado da aplicação.
    let cd_state = web::Data::new(state::CdState {
        inventory: Arc::new(Mutex::new(initial_inventory)),
        service_discovery_url: service_discovery_url.clone(),
        hub_url: hub_url.clone(),
        http_client: Client::new(),
        own_id: cd_id.clone(),
    });

    // 4. Registra este serviço no Service Discovery.
    let client_for_register = Client::new();
    let service_info = ServiceInfo {
        id: cd_id.clone(),
        ip: ip.clone(),
        port,
        last_heartbeat: chrono::Utc::now(),
    };
    let discovery_url_register = format!("{}/register", service_discovery_url);

    match client_for_register
        .post(&discovery_url_register)
        .json(&service_info)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            println!(
                "[{}] Registered with Service Discovery at {}:{}",
                cd_id, ip, port
            );
        }
        Ok(resp) => {
            eprintln!(
                "[{}] Failed to register with Service Discovery: Status {} - {:?}",
                cd_id,
                resp.status(),
                resp.text().await
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!(
                "[{}] Failed to register with Service Discovery: {}",
                cd_id, e
            );
            std::process::exit(1);
        }
    }

    // 5. Dispara a tarefa de fundo para enviar heartbeats.
    let heartbeat_state = cd_state.clone();
    tokio::spawn(client::send_heartbeat(heartbeat_state));

    // Dispara tarefas de fundo para simular a requisição de produtos.
    let request_state = cd_state.clone();
    tokio::spawn(async move {
        if request_state.own_id == "cd_alpha" {
            sleep(Duration::from_secs(5)).await;
            if let Err(e) = client::request_product_from_system(
                request_state.clone(),
                "celulares".to_string(),
                12,
            )
            .await
            {
                eprintln!(
                    "[{}] Error fulfilling request for celulares: {}",
                    request_state.own_id, e
                );
            }
        }
        if request_state.own_id == "cd_beta" {
            sleep(Duration::from_secs(7)).await;
            if let Err(e) = client::request_product_from_system(
                request_state.clone(),
                "canetas".to_string(),
                50,
            )
            .await
            {
                eprintln!(
                    "[{}] Error fulfilling request for canetas: {}",
                    request_state.own_id, e
                );
            }
        }
    });

    println!("[{}] CD Service running on http://{}:{}", cd_id, ip, port);

    // 6. Inicia o servidor HTTP com os endpoints da API do CD.
    HttpServer::new(move || {
        App::new()
            .app_data(cd_state.clone())
            .service(
                web::resource("/inventory/{product_code}").get(handlers::get_product_inventory),
            )
            .service(web::resource("/transfer_product").post(handlers::transfer_product))
            .service(web::resource("/receive_product").post(handlers::receive_product))
    })
    .bind(format!("{}:{}", ip, port))?
    .run()
    .await
}
