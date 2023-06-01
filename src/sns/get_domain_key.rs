use crate::{trace, ErrorType};
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::derivation::get_domain_key;
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

pub async fn process(_rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    let domain_key =
        get_domain_key(&params.domain, false).map_err(|e| trace!(ErrorType::InvalidDomain, e))?;
    Ok(serde_json::to_value(domain_key.to_string()).map_err(|e| trace!(ErrorType::Generic, e)))?
}
