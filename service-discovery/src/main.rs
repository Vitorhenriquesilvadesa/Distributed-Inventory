use actix_web::{web, App, HttpServer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod handlers;
mod state;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            handlers::register_service,
            handlers::lookup_service,
            handlers::lookup_all_services,
            handlers::heartbeat,
        ),
        components(
            schemas(common_models::ServiceInfo, common_models::ServiceInfoLookup)
        ),
        tags(
            (name = "Service Discovery", description = "Service registration and lookup endpoints")
        ),
        info(
            title = "Service Discovery API",
            version = "1.0.0",
            description = "Manages the registration, lookup, and health of all microservices.",
        )
    )]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();

    let app_state = web::Data::new(state::AppState {
        registered_services: Arc::new(Mutex::new(HashMap::new())),
    });

    let cleanup_state = app_state.clone();
    tokio::spawn(handlers::cleanup_inactive_services(cleanup_state));

    let ip = "127.0.0.1";
    let port = 8080;

    println!("Service Discovery running on http://{}:{}", ip, port);
    println!("Swagger UI available at http://{}:{}/swagger-ui/", ip, port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(web::resource("/register").post(handlers::register_service))
            .service(web::resource("/lookup/{id}").get(handlers::lookup_service))
            .service(web::resource("/lookup_all").get(handlers::lookup_all_services))
            .service(web::resource("/heartbeat/{id}").post(handlers::heartbeat))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", openapi.clone()),
            )
    })
    .bind(format!("{}:{}", ip, port))?
    .run()
    .await
}