use std::str::FromStr;

use crate::{trace, ErrorType};
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::non_blocking::resolve;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[derive(Deserialize)]
pub struct Params {
    domain_key: String,
}

impl Params {
    pub fn deserialize(value: Value) -> Result<Self, crate::Error> {
        if let Some(v) = value.as_array() {
            let domain_key = v
                .get(0)
                .ok_or(trace!(ErrorType::MissingParameters))?
                .as_str()
                .ok_or(trace!(ErrorType::InvalidParameters))?
                .to_owned();
            Ok(Self { domain_key })
        } else {
            serde_json::from_value(value).map_err(|e| trace!(ErrorType::InvalidParameters, e))
        }
    }
}

pub async fn process(rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    let domain_key = Pubkey::from_str(&params.domain_key)
        .map_err(|e| trace!(ErrorType::InvalidParameters, e))?;

    // TODO fix resolve_reverse to return Result<Option<..>, ..>
    let reversed = resolve::resolve_reverse(&rpc_client, &domain_key)
        .await
        .map_err(|e| trace!(ErrorType::DomainNotFound, e))?;
    Ok(serde_json::to_value(reversed).map_err(|e| trace!(ErrorType::Generic, e)))?
}
