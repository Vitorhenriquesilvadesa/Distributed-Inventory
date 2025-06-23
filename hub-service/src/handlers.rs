use super::state::AppState;
use actix_web::{web, HttpResponse, Responder};
use common_models::{Product, ProductAvailability, ServiceInfoLookup};
use futures::future::join_all;

/// Registra um novo tipo de produto no catálogo central.
#[utoipa::path(
    post,
    path = "/products",
    request_body = Product,
    responses(
        (status = 200, description = "Product registered successfully", body = String, example = json!("Product laptop registered successfully")),
    ),
    tag = "Hub Service"
)]
pub async fn register_product(
    product: web::Json<Product>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut catalog = data.products_catalog.lock().unwrap();
    let product_code = product.code.clone();
    catalog.insert(product_code.clone(), product.into_inner());
    println!("Registered product: {}", product_code);
    HttpResponse::Ok().body(format!("Product {} registered successfully", product_code))
}

/// Consulta os detalhes de um produto específico no catálogo.
#[utoipa::path(
    get,
    path = "/products/{product_code}",
    params(
        ("product_code" = String, Path, description = "Code of the product to lookup")
    ),
    responses(
        (status = 200, description = "Product details found", body = Product),
        (status = 404, description = "Product not found in catalog", body = String, example = json!("Product mouse not found in catalog"))
    ),
    tag = "Hub Service"
)]
pub async fn get_product_details(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let product_code = path.into_inner();
    let catalog = data.products_catalog.lock().unwrap();

    if let Some(product) = catalog.get(&product_code) {
        HttpResponse::Ok().json(product)
    } else {
        HttpResponse::NotFound().body(format!("Product {} not found in catalog", product_code))
    }
}

/// Encontra quais CDs possuem uma quantidade mínima de um produto.
#[utoipa::path(
    get,
    path = "/who_has_product/{product_code}/{quantity_needed}",
    params(
        ("product_code" = String, Path, description = "Code of the desired product"),
        ("quantity_needed" = u32, Path, description = "Minimum quantity needed")
    ),
    responses(
        (status = 200, description = "List of CDs that have the product", body = Vec<ProductAvailability>),
        (status = 404, description = "Product not found in any CD", body = String),
        (status = 500, description = "Failed to query Service Discovery")
    ),
    tag = "Hub Service"
)]
pub async fn who_has_product(
    path: web::Path<(String, u32)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (product_code, quantity_needed) = path.into_inner();
    let client = &data.http_client;
    let service_discovery_url = &data.service_discovery_url;

    let lookup_all_cds_url = format!("{}/lookup_all", service_discovery_url);
    let cd_ids_res = client.get(lookup_all_cds_url).send().await;

    let cd_ids: Vec<String> = match cd_ids_res {
        Ok(resp) => resp.json().await.unwrap_or_else(|_| {
            eprintln!("Failed to parse all CD IDs from Service Discovery, returning empty Vec.");
            Vec::new()
        }),
        Err(e) => {
            eprintln!("Error getting all CD IDs from Service Discovery: {}", e);
            return HttpResponse::InternalServerError().body("Failed to query Service Discovery");
        }
    };

    if cd_ids.is_empty() {
        return HttpResponse::NotFound().body("No CDs registered in Service Discovery.");
    }

    let mut futures = Vec::new();

    for cd_id in cd_ids {
        let client = client.clone();
        let service_discovery_url = service_discovery_url.clone();
        let product_code = product_code.clone();
        let catalog_data = data.products_catalog.clone();

        futures.push(async move {
            let service_info_res = client
                .get(format!("{}/lookup/{}", service_discovery_url, cd_id))
                .send()
                .await;

            let service_info: ServiceInfoLookup = match service_info_res {
                Ok(resp) => resp.json().await.ok()?,
                Err(_) => {
                    eprintln!(
                        "Could not lookup service {}: Network error or not found",
                        cd_id
                    );
                    return None;
                }
            };

            let cd_url = format!(
                "http://{}:{}/inventory/{}",
                service_info.ip, service_info.port, product_code
            );
            let product_res = client.get(&cd_url).send().await;

            match product_res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let product_in_cd: Product = resp.json().await.ok()?;
                        if let Some(quantity) = product_in_cd.quantity {
                            if quantity >= quantity_needed {
                                let product_details = catalog_data
                                    .lock()
                                    .unwrap()
                                    .get(&product_code)
                                    .cloned()
                                    .unwrap_or_else(|| product_in_cd.clone());
                                return Some(ProductAvailability {
                                    cd_id: cd_id.clone(),
                                    quantity_available: quantity,
                                    product_info: product_details,
                                });
                            }
                        }
                    }
                    None
                }
                Err(e) => {
                    eprintln!(
                        "Error querying CD {} for product {}: {}",
                        cd_id, product_code, e
                    );
                    None
                }
            }
        });
    }

    let results: Vec<Option<ProductAvailability>> = join_all(futures).await;
    let available_cds: Vec<ProductAvailability> = results.into_iter().flatten().collect();

    if available_cds.is_empty() {
        HttpResponse::NotFound().body(format!(
            "Product {} with quantity {} not found in any CD",
            product_code, quantity_needed
        ))
    } else {
        HttpResponse::Ok().json(available_cds)
    }
}