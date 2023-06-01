use crate::{trace, ErrorType};
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::derivation::get_domain_key;
use solana_client::nonblocking::rpc_client::RpcClient;

#[derive(Deserialize)]
pub struct Params {
    domain: String,
    record: String,
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
            let record = v
                .get(1)
                .ok_or(trace!(ErrorType::MissingParameters))?
                .as_str()
                .ok_or(trace!(ErrorType::InvalidParameters))?
                .to_owned();
            Ok(Self { domain, record })
        } else {
            serde_json::from_value(value).map_err(|e| trace!(ErrorType::InvalidParameters, e))
        }
    }
}

pub async fn process(_rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    let domain_record_key = get_domain_key(&format!("{}.{}", params.record, params.domain), true)
        .map_err(|e| trace!(ErrorType::InvalidDomain, e))?;
    Ok(serde_json::to_value(domain_record_key.to_string())
        .map_err(|e| trace!(ErrorType::Generic, e)))?
}
