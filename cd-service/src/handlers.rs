// cd-service/src/handlers.rs
use super::state::CdState;
use actix_web::{web, HttpResponse, Responder};
use common_models::{Product, TransferRequest};

// GET /inventory/{product_code}: Retorna a quantidade e detalhes de um produto no inventário local.
pub async fn get_product_inventory(
    path: web::Path<String>,
    data: web::Data<CdState>,
) -> impl Responder {
    let product_code = path.into_inner();
    let inventory = data.inventory.lock().unwrap();

    if let Some(product) = inventory.get(&product_code) {
        HttpResponse::Ok().json(product)
    } else {
        HttpResponse::NotFound().body(format!("Product {} not found in this CD", product_code))
    }
}

// POST /transfer_product: Recebe um pedido de transferência de outro CD.
pub async fn transfer_product(
    transfer_req: web::Json<TransferRequest>,
    data: web::Data<CdState>,
) -> impl Responder {
    let mut inventory = data.inventory.lock().unwrap();
    if let Some(product) = inventory.get_mut(&transfer_req.product_code) {
        // CORREÇÃO: Acessar a quantidade usando .unwrap_or(0) ou match
        // Como estamos lidando com um inventário, esperamos que quantity seja Some(u32)
        let current_quantity = product.quantity.unwrap_or(0); // Assume 0 se for None, o que não deveria acontecer no inventário do CD
        if current_quantity >= transfer_req.quantity {
            product.quantity = Some(current_quantity - transfer_req.quantity);
            println!(
                "[{}] Transferred {} of {} to {}",
                data.own_id,
                transfer_req.quantity,
                transfer_req.product_code,
                transfer_req.requester_cd_id
            );
            HttpResponse::Ok().body("Transfer successful")
        } else {
            HttpResponse::BadRequest().body(format!(
                "Not enough quantity of {} in {} for transfer. Has {}, requested {}",
                transfer_req.product_code, data.own_id, current_quantity, transfer_req.quantity
            ))
        }
    } else {
        HttpResponse::NotFound().body(format!(
            "Product {} not found in {} for transfer",
            transfer_req.product_code, data.own_id
        ))
    }
}

// POST /receive_product: Recebe produtos de outro CD (simplesmente adiciona ao inventário).
pub async fn receive_product(
    product_data: web::Json<Product>,
    data: web::Data<CdState>,
) -> impl Responder {
    let mut inventory = data.inventory.lock().unwrap();
    let product_code = product_data.code.clone();
    let quantity_received = product_data.quantity.unwrap_or(0); // CORREÇÃO: Tratar Option<u32>

    inventory
        .entry(product_code.clone())
        .and_modify(|p| p.quantity = Some(p.quantity.unwrap_or(0) + quantity_received)) // CORREÇÃO
        .or_insert(Product {
            code: product_data.code.clone(),
            name: product_data.name.clone(),
            price: product_data.price,
            quantity: Some(quantity_received),
        });

    println!(
        "[{}] Received {} of {}",
        data.own_id, quantity_received, product_code
    );
    HttpResponse::Ok().body("Product received successfully")
}
