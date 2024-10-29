use std::str::FromStr;

use actix_web::{get, post, web, Scope};
use anyhow::anyhow;
use anyhow_to_actix_error::Result;
use bitcoincore_rpc::{bitcoin::Address, json::GetNetworkInfoResult, RpcApi};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[get("/network")]
async fn get_network_info(data: web::Data<AppState>) -> Result<web::Json<GetNetworkInfoResult>> {
    tracing::info!("Get infos");

    let network_infos = data
        .rpc_client
        .lock()
        .await
        .get_network_info()
        .map_err(|e| anyhow!("sexe {}", e))?;

    Ok(web::Json(network_infos))
}

#[derive(Serialize)]
struct TxResponse {
    txid: String,
}

#[derive(Deserialize)]
struct TransactionRequest {
    transaction: String,
}

#[post("/tx")]
async fn publish_transaction(
    data: web::Data<AppState>,
    body: web::Json<TransactionRequest>,
) -> Result<web::Json<TxResponse>> {
    let signed_tx = body.transaction.clone();

    tracing::info!("Publish transaction {}", signed_tx);

    let rpc = data.rpc_client.lock().await;
    let txid = rpc
        .send_raw_transaction(signed_tx)
        .map_err(|e| anyhow!("sexe {}", e))?;

    Ok(web::Json(TxResponse {
        txid: txid.to_string(),
    }))
}

#[derive(Serialize)]
struct UtxoResponse {
    txid: String,
    vout: u32,
    amount: f64,
    address: String,
}

#[get("address/{address}/utxo")]
async fn get_previous_transaction_by_address(
    data: web::Data<AppState>,
    address: web::Path<String>,
) -> Result<web::Json<Vec<UtxoResponse>>> {
    let address = address.into_inner();
    tracing::info!("Get previous transaction of {}", address);

    let rpc = data.rpc_client.lock().await;
    let address = Address::from_str(&address)
        .map_err(|e| anyhow!("sexe {}", e))?
        .require_network(bitcoincore_rpc::bitcoin::Network::Regtest)
        .map_err(|e| anyhow!("sexe {}", e))?;

    let utxos = rpc
        .list_unspent(Some(0), Some(9999999), Some(&[&address]), None, None)
        .map_err(|e| anyhow!("sexe {}", e))?;

    let utxos: Vec<UtxoResponse> = utxos
        .into_iter()
        .map(|utxo| UtxoResponse {
            txid: utxo.txid.to_string(),
            vout: utxo.vout,
            amount: utxo.amount.to_btc(),
            address: utxo.address.map_or_else(
                || "unknown".to_string(),
                |addr| {
                    addr.require_network(bitcoincore_rpc::bitcoin::Network::Regtest)
                        .unwrap()
                        .to_string()
                },
            ),
        })
        .collect();

    Ok(web::Json(utxos))
}

pub fn service() -> Scope {
    web::scope("/regtest/api")
        .service(get_network_info)
        .service(publish_transaction)
        .service(get_previous_transaction_by_address)
}
