use crate::{append_trace, trace, ErrorType};
use base64::Engine;
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::{
    derivation::get_domain_key,
    record::{Record, RecordVersion},
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use spl_name_service::state::NameRecordHeader;

use super::{
    get_opt_int_from_value_array, get_opt_string_from_value_array, get_string_from_value_array,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    domain: String,
    record: Option<String>,
    record_version: Option<u8>,
}

impl Params {
    pub fn deserialize(value: Value) -> Result<Self, crate::Error> {
        if let Some(v) = value.as_array() {
            let domain = get_string_from_value_array(v, 0).map_err(|e| append_trace!(e))?;
            let record = get_opt_string_from_value_array(v, 1).map_err(|e| append_trace!(e))?;
            let record_version =
                get_opt_int_from_value_array(v, 2).map_err(|e| append_trace!(e))?;
            Ok(Self {
                domain,
                record,
                record_version,
            })
        } else {
            serde_json::from_value(value).map_err(|e| trace!(ErrorType::InvalidParameters, e))
        }
    }
}

pub async fn process(rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    let Params {
        domain,
        record,
        record_version,
    } = params;
    let record_version = match record_version {
        Some(1) | None => RecordVersion::V1,
        Some(2) => RecordVersion::V2,
        _ => return Err(trace!(ErrorType::InvalidRecord)),
    };
    let domain_key = match record {
        None => get_domain_key(&domain),
        Some(r) => {
            let record =
                Record::try_from_str(&r).map_err(|e| trace!(ErrorType::InvalidRecord, e))?;
            sns_sdk::record::get_record_key(&domain, record, record_version)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn integrated_test_0() {
        use crate::sns::{Method, RpcMessage, RpcResponseOk, JSON_RPC};
        let endpoint = std::env::var("TEST_QUICKNODE_ENDPOINT").unwrap();
        let client = reqwest::Client::new();
        let message = RpcMessage {
            jsonrpc: JSON_RPC.to_owned(),
            method: Method::GetDomainData,
            params: serde_json::to_value(["bonfida.sol", "github"]).unwrap(),
            id: serde_json::to_value(42u8).unwrap(),
        };
        eprintln!("{}", serde_json::to_string_pretty(&message).unwrap());
        let post_request = client.post(&endpoint).json(&message).build().unwrap();
        let response = client.execute(post_request).await.unwrap();
        eprintln!("{:#?}", response);
        if response.status().is_success() {
            let result: RpcResponseOk<String> = response.json().await.unwrap();
            let value = result.result.as_str().unwrap();
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(value)
                .unwrap();
            let s = std::str::from_utf8(&bytes[..27]).unwrap();
            assert_eq!(s, "https://github.com/Bonfida/");
            assert_eq!(bytes.len(), 2096 - NameRecordHeader::LEN);
        } else {
            let text = response.text().await.unwrap();
            eprintln!("Error body:\n {text}");
            panic!()
        }
    }
}
