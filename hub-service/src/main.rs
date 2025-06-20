use actix_web::{web, App, HttpServer};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod handlers;
mod state;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let service_discovery_url = "http://127.0.0.1:8080".to_string();
    let ip = "127.0.0.1";
    let port = 8082;

    let app_state = web::Data::new(state::AppState {
        products_catalog: Arc::new(Mutex::new(HashMap::new())),
        service_discovery_url: service_discovery_url.clone(),
        http_client: Client::new(),
    });

    println!("Hub Service running on http://{}:{}", ip, port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(web::resource("/products").post(handlers::register_product))
            .service(web::resource("/products/{product_code}").get(handlers::get_product_details))
            .service(
                web::resource("/who_has_product/{product_code}/{quantity_needed}")
                    .get(handlers::who_has_product),
            )
    })
    .bind(format!("{}:{}", ip, port))?
    .run()
    .await
}
