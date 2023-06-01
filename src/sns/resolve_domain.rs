use crate::{trace, ErrorType};
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
            let domain = v
                .get(0)
                .ok_or(trace!(ErrorType::MissingParameters))?
                .as_str()
                .ok_or(trace!(ErrorType::InvalidParameters))?
                .to_owned();
            Ok(Self { domain })
        } else {
            serde_json::from_value(value).map_err(|e| trace!(ErrorType::InvalidParameters, e))
        }
    }
}

pub async fn process(rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    let resolved = resolve::resolve_owner(&rpc_client, &params.domain)
        .await
        .map_err(|e| trace!(ErrorType::Generic, e))?;
    Ok(serde_json::to_value(resolved.to_string()).map_err(|e| trace!(ErrorType::Generic, e)))?
}
