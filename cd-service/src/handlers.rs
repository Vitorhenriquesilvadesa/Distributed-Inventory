// cd-service/src/handlers.rs
use super::state::CdState;
use actix_web::{web, HttpResponse, Responder};
use common_models::{Product, TransferRequest};

/// **GET /inventory/{product_code}**
///
/// Retorna a quantidade e os detalhes de um produto específico no inventário local do CD.
///
/// # Parâmetros de Rota
///
/// * `product_code`: O código do produto a ser consultado (ex: "garrafas").
///
/// # Resposta de Sucesso (200 OK)
///
/// ```json
/// {
///   "code": "garrafas",
///   "name": "Garrafas de Água",
///   "price": 2.50,
///   "quantity": 50
/// }
/// ```
///
/// # Resposta de Erro (404 Not Found)
///
/// ```text
/// Product garrafas not found in this CD
/// ```
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

/// **POST /transfer_product**
///
/// Processa uma solicitação de transferência de produto vinda de outro CD.
///
/// Se o CD atual tiver a quantidade solicitada em estoque, ele deduz essa quantidade
/// de seu inventário e retorna uma resposta de sucesso. Este endpoint é chamado
/// pelo CD que *precisa* do produto.
///
/// # Corpo da Requisição (JSON)
///
/// ```json
/// {
///   "product_code": "celulares",
///   "quantity": 5,
///   "requester_cd_id": "cd_alpha"
/// }
/// ```
///
/// # Resposta de Sucesso (200 OK)
///
/// ```text
/// Transfer successful
/// ```
///
/// # Respostas de Erro
///
/// * **400 Bad Request**: Se a quantidade em estoque for insuficiente.
/// * **404 Not Found**: Se o produto não existir no inventário deste CD.
pub async fn transfer_product(
    transfer_req: web::Json<TransferRequest>,
    data: web::Data<CdState>,
) -> impl Responder {
    let mut inventory = data.inventory.lock().unwrap();
    if let Some(product) = inventory.get_mut(&transfer_req.product_code) {
        let current_quantity = product.quantity.unwrap_or(0);
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

/// **POST /receive_product**
///
/// Adiciona um produto e sua quantidade ao inventário local.
///
/// Este endpoint serve como um mecanismo para registrar a entrada de produtos no
/// inventário do CD. Se o produto já existe, a quantidade é somada;
/// caso contrário, um novo registro de produto é criado.
///
/// # Corpo da Requisição (JSON)
///
/// ```json
/// {
///   "code": "celulares",
///   "name": "Smartphones X",
///   "price": 1200.0,
///   "quantity": 5
/// }
/// ```
///
/// # Resposta de Sucesso (200 OK)
///
/// ```text
/// Product received successfully
/// ```
pub async fn receive_product(
    product_data: web::Json<Product>,
    data: web::Data<CdState>,
) -> impl Responder {
    let mut inventory = data.inventory.lock().unwrap();
    let product_code = product_data.code.clone();
    let quantity_received = product_data.quantity.unwrap_or(0);

    inventory
        .entry(product_code.clone())
        .and_modify(|p| p.quantity = Some(p.quantity.unwrap_or(0) + quantity_received))
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
