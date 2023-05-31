use std::str::FromStr;

use crate::trace;
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::non_blocking::resolve;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[derive(Deserialize)]
pub struct Params {
    owner: String,
}

impl Params {
    pub fn deserialize(value: Value) -> Result<Self, crate::Error> {
        if let Some(v) = value.as_array() {
            let owner = v[0]
                .as_str()
                .ok_or(trace!(crate::ErrorType::InvalidParameters))?
                .to_owned();
            Ok(Self { owner })
        } else {
            serde_json::from_value(value)
                .map_err(|e| trace!(crate::ErrorType::InvalidParameters, e))
        }
    }
}

pub async fn process(rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    let owner = Pubkey::from_str(&params.owner)
        .map_err(|e| trace!(crate::ErrorType::InvalidParameters, e))?;
    let domains: Vec<String> = resolve::get_domains_owner(&rpc_client, owner)
        .await
        .map_err(|e| trace!(crate::ErrorType::Generic, e))?
        .into_iter()
        .map(|k| k.to_string())
        .collect::<Vec<_>>();
    Ok(serde_json::to_value(domains).map_err(|e| trace!(crate::ErrorType::Generic, e)))?
}
