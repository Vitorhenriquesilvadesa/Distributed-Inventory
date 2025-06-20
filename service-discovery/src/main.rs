use actix_web::{web, App, HttpServer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod handlers;
mod state;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(state::AppState {
        registered_services: Arc::new(Mutex::new(HashMap::new())),
    });

    let cleanup_state = app_state.clone();
    tokio::spawn(handlers::cleanup_inactive_services(cleanup_state));

    let ip = "127.0.0.1";
    let port = 8080;

    println!("Service Discovery running on http://{}:{}", ip, port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(web::resource("/register").post(handlers::register_service))
            .service(web::resource("/lookup/{id}").get(handlers::lookup_service))
            .service(web::resource("/lookup_all").get(handlers::lookup_all_services))
            .service(web::resource("/heartbeat/{id}").post(handlers::heartbeat))
    })
    .bind(format!("{}:{}", ip, port))?
    .run()
    .await
}
