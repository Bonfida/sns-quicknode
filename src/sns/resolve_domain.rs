use crate::{db::DbConnector, sns::get_rpc_client, trace};
use actix_web::{web, HttpRequest};
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::non_blocking::resolve;
use solana_client::nonblocking::rpc_client::RpcClient;

#[derive(Deserialize)]
pub struct Params {
    domain: String,
}

impl Params {
    pub fn deserialize(value: Value) -> Result<Self, crate::Error> {
        if let Some(v) = value.as_array() {
            let domain = v[0]
                .as_str()
                .ok_or(trace!(crate::ErrorType::InvalidParameters))?
                .to_owned();
            Ok(Self { domain })
        } else {
            serde_json::from_value(value).map_err(|e| trace!(crate::ErrorType::InvalidParameters))
        }
    }
}

pub async fn run(rpc_client: RpcClient, params: Params) -> Result<Value, crate::Error> {
    let resolved = resolve::resolve_owner(&rpc_client, &params.domain)
        .await
        .map_err(|e| trace!(crate::ErrorType::Generic, e))?;
    Ok(serde_json::to_value(resolved.to_string())
        .map_err(|e| trace!(crate::ErrorType::Generic, e)))?
}

pub async fn route(
    request: HttpRequest,
    domain: web::Path<String>,
    db: web::Data<DbConnector>,
) -> Result<String, crate::Error> {
    let rpc_client = get_rpc_client(&db, &request).await?;
    let resolved = resolve::resolve_owner(&rpc_client, &domain)
        .await
        .map_err(|e| trace!(crate::ErrorType::Generic, e))?;
    Ok(resolved.to_string())
}
