use crate::{append_trace, trace, ErrorType};
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::derivation::get_reverse_key;
use solana_client::nonblocking::rpc_client::RpcClient;

use super::get_string_from_value_array;

#[derive(Deserialize)]
pub struct Params {
    domain: String,
}

impl Params {
    pub fn deserialize(value: Value) -> Result<Self, crate::Error> {
        if let Some(v) = value.as_array() {
            let domain = get_string_from_value_array(v, 0).map_err(|e| append_trace!(e))?;
            Ok(Self { domain })
        } else {
            serde_json::from_value(value).map_err(|e| trace!(ErrorType::InvalidParameters, e))
        }
    }
}

pub async fn process(_rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    let reverse_domain_key =
        get_reverse_key(&params.domain).map_err(|e| trace!(ErrorType::InvalidDomain, e))?;
    Ok(serde_json::to_value(reverse_domain_key.to_string())
        .map_err(|e| trace!(ErrorType::Generic, e)))?
}
