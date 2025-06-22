use super::state::CdState;
use actix_web::web;
use common_models::{
    Product, ProductAvailability, ServiceInfoLookup, TransferRequest,
};
use tokio::time::{sleep, Duration};

/// Orquestra a busca e aquisição de um produto de outros CDs no sistema.
///
/// Esta função encapsula a lógica principal de um `CD Service` quando ele precisa
/// de um produto que não possui em estoque ou cuja quantidade é insuficiente. O processo
/// envolve múltiplos passos e interações com os outros componentes do sistema:
///
/// 1.  **Verificação Local:** Primeiramente, a função consulta o inventário local do próprio CD.
/// 2.  **Consulta ao Hub:** Se o estoque for insuficiente, calcula a quantidade faltante e
///     envia uma requisição ao `Hub Service` (`GET /who_has_product/{code}/{quantity}`)
///     para descobrir quais outros CDs possuem o produto na quantidade necessária.
/// 3.  **Localização via Service Discovery:** Ao receber a lista de CDs disponíveis do Hub,
///     a função seleciona um fornecedor e consulta o `Service Discovery` (`GET /lookup/{id}`)
///     para obter o endereço de rede do CD fornecedor.
/// 4.  **Transferência P2P:** Com o endereço do fornecedor, inicia uma transferência
///     direta (P2P), enviando uma requisição `POST /transfer_product` para o CD fornecedor.
/// 5.  **Atualização de Inventário:** Se a transferência for bem-sucedida, a função atualiza
///     o inventário local, adicionando a quantidade recebida do produto.
///
/// # Arguments
///
/// * `state`: Um `web::Data<CdState>` que contém o estado compartilhado do serviço.
/// * `product_code`: O código do produto que precisa ser adquirido (ex: "celulares").
/// * `quantity_needed`: A quantidade total do produto que o CD deseja possuir.
///
/// # Returns
///
/// * `Ok(())` - Se a necessidade do produto for satisfeita com sucesso.
/// * `Err(String)` - Retorna uma descrição textual do erro em caso de falha.
///
/// # Panics
///
/// * A função utiliza `state.inventory.lock().unwrap()`. Este método causará um pânico (panic)
///   se o `Mutex` que protege o inventário estiver "envenenado".
///
/// # Side Effects
///
/// * Realiza requisições de rede para outros serviços (Hub, Service Discovery, outros CDs).
/// * Modifica o estado do inventário local (`state.inventory`) ao receber um produto.
pub async fn request_product_from_system(
    state: web::Data<CdState>,
    product_code: String,
    quantity_needed: u32,
) -> Result<(), String> {
    println!(
        "[{}] Requesting {} of product {}",
        state.own_id, quantity_needed, product_code
    );

    let mut quantity_to_find = quantity_needed;

    {
        let mut current_inventory = state.inventory.lock().unwrap();
        if let Some(product) = current_inventory.get_mut(&product_code) {
            let product_current_quantity = product.quantity.unwrap_or(0);
            if product_current_quantity >= quantity_needed {
                println!(
                    "[{}] Already have enough of {}. Quantity: {}",
                    state.own_id, product_code, product_current_quantity
                );
                return Ok(());
            } else {
                quantity_to_find -= product_current_quantity;
                println!(
                    "[{}] Only have {} of {}. Need {} more.",
                    state.own_id, product_current_quantity, product_code, quantity_to_find
                );
            }
        } else {
            println!(
                "[{}] Don't have any of {}. Need {} total.",
                state.own_id, product_code, quantity_to_find
            );
        }
    }

    let client = &state.http_client;
    let hub_who_has_url = format!(
        "{}/who_has_product/{}/{}",
        state.hub_url, product_code, quantity_to_find
    );
    println!("[{}] Querying Hub: {}", state.own_id, hub_who_has_url);

    let response = client
        .get(&hub_who_has_url)
        .send()
        .await
        .map_err(|e| format!("Failed to query Hub: {}", e))?;

    let status = response.status();

    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Hub returned error {}: {}", status, error_body));
    }

    let available_cds: Vec<ProductAvailability> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Hub response: {}", e))?;

    if available_cds.is_empty() {
        return Err(format!(
            "No CD found with product {} (quantity {})",
            product_code, quantity_to_find
        ));
    }

    println!(
        "[{}] Hub found these CDs with {}: {:?}",
        state.own_id, product_code, available_cds
    );

    for cd_availability in available_cds {
        if cd_availability.quantity_available >= quantity_to_find {
            let target_cd_id = cd_availability.cd_id;
            println!(
                "[{}] Trying to get {} of {} from CD: {}",
                state.own_id, quantity_to_find, product_code, target_cd_id
            );

            let lookup_url = format!("{}/lookup/{}", state.service_discovery_url, target_cd_id);
            let target_cd_info: ServiceInfoLookup = client
                .get(&lookup_url)
                .send()
                .await
                .map_err(|e| format!("Failed to lookup target CD {}: {}", target_cd_id, e))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse target CD info: {}", e))?;

            let transfer_url = format!(
                "http://{}:{}/transfer_product",
                target_cd_info.ip, target_cd_info.port
            );
            println!(
                "[{}] Sending transfer request to {}: {}",
                state.own_id, target_cd_id, transfer_url
            );

            let transfer_request_body = TransferRequest {
                product_code: product_code.clone(),
                quantity: quantity_to_find,
                requester_cd_id: state.own_id.clone(),
            };

            let transfer_response = client
                .post(&transfer_url)
                .json(&transfer_request_body)
                .send()
                .await
                .map_err(|e| {
                    format!("Failed to send transfer request to {}: {}", target_cd_id, e)
                })?;

            let transfer_status = transfer_response.status();

            if transfer_status.is_success() {
                println!(
                    "[{}] Successfully transferred {} of {} from {}",
                    state.own_id, quantity_to_find, product_code, target_cd_id
                );

                {
                    let mut inventory = state.inventory.lock().unwrap();
                    inventory
                        .entry(product_code.clone())
                        .and_modify(|p| {
                            p.quantity = Some(p.quantity.unwrap_or(0) + quantity_to_find)
                        })
                        .or_insert(Product {
                            code: product_code.clone(),
                            name: cd_availability.product_info.name,
                            price: cd_availability.product_info.price,
                            quantity: Some(quantity_to_find),
                        });
                    println!(
                        "[{}] Current inventory for {}: {:?}",
                        state.own_id,
                        product_code,
                        inventory.get(&product_code)
                    );
                }

                return Ok(());
            } else {
                let error_body = transfer_response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                eprintln!(
                    "[{}] Failed to transfer from {}: {}: {}",
                    state.own_id, target_cd_id, transfer_status, error_body
                );
            }
        }
    }

    Err(format!(
        "Could not fulfill request for {} of {} from any available CD",
        quantity_needed, product_code
    ))
}

/// Executa uma tarefa de fundo que envia sinais de "heartbeat" para o Service Discovery.
///
/// Esta função é projetada para ser executada em uma `tokio::task` separada e entra
/// em um loop infinito. A cada 10 segundos, ela envia uma requisição `POST` para o
/// endpoint `/heartbeat/{id}` do `Service Discovery`, sinalizando que o `CD Service`
/// atual ainda está ativo e funcional.
///
/// # Arguments
///
/// * `state`: O estado compartilhado `CdState`, de onde a função obtém a URL do
///   `Service Discovery`, o ID do próprio CD e o cliente HTTP.
///
/// # Returns
///
/// * Esta função nunca retorna, pois opera em um `loop` infinito.
///
/// # Side Effects
///
/// * Executa continuamente em segundo plano.
/// * Envia uma requisição de rede para o `Service Discovery` a cada 10 segundos.
pub async fn send_heartbeat(state: web::Data<CdState>) {
    let client = &state.http_client;
    let heartbeat_url = format!("{}/heartbeat/{}", state.service_discovery_url, state.own_id);
    loop {
        sleep(Duration::from_secs(10)).await;
        match client.post(&heartbeat_url).send().await {
            Ok(_) => {}
            Err(e) => eprintln!("[{}] Failed to send heartbeat: {}", state.own_id, e),
        }
    }
}
