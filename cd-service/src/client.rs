use super::state::CdState;
use actix_web::web;
use common_models::{
    Product, ProductAvailability, ServiceInfoLookup, TransferRequest,
};
use tokio::time::{sleep, Duration};

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
