use crate::{append_trace, trace, ErrorType};
use base64::Engine;
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::derivation::get_domain_key;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use spl_name_service::state::NameRecordHeader;

use super::{get_opt_string_from_value_array, get_string_from_value_array};

#[derive(Deserialize)]
pub struct Params {
    domain: String,
    record: Option<String>,
}

impl Params {
    pub fn deserialize(value: Value) -> Result<Self, crate::Error> {
        if let Some(v) = value.as_array() {
            let domain = get_string_from_value_array(v, 0).map_err(|e| append_trace!(e))?;
            let record = get_opt_string_from_value_array(v, 1).map_err(|e| append_trace!(e))?;
            Ok(Self { domain, record })
        } else {
            serde_json::from_value(value).map_err(|e| trace!(ErrorType::InvalidParameters, e))
        }
    }
}

pub async fn process(rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    let Params { domain, record } = params;
    let domain_key = match record {
        None => get_domain_key(&domain, false),
        Some(r) => get_domain_key(&format!("{}.{}", r, domain), true),
    }
    .map_err(|e| trace!(ErrorType::InvalidDomain, e))?;
    let account = rpc_client
        .get_account_with_commitment(&domain_key, rpc_client.commitment())
        .await
        .map_err(|e| trace!(ErrorType::SolanaRpcError, e))?
        .value;
    let data = account.map(|a| {
        base64::engine::general_purpose::STANDARD.encode(&a.data[NameRecordHeader::LEN..])
    });
    Ok(serde_json::to_value(data).map_err(|e| trace!(ErrorType::Generic, e)))?
}
