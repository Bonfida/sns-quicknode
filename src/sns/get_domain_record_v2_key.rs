use crate::{append_trace, trace, ErrorType};
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::record::{get_record_key, Record, RecordVersion};
use solana_client::nonblocking::rpc_client::RpcClient;

use super::get_string_from_value_array;

#[derive(Deserialize)]
pub struct Params {
    pub domain: String,
    pub record: String,
}

impl Params {
    pub fn deserialize(value: Value) -> Result<Self, crate::Error> {
        if let Some(v) = value.as_array() {
            let domain = get_string_from_value_array(v, 0).map_err(|e| append_trace!(e))?;
            let record = get_string_from_value_array(v, 1).map_err(|e| append_trace!(e))?;
            Ok(Self { domain, record })
        } else {
            serde_json::from_value(value).map_err(|e| trace!(ErrorType::InvalidParameters, e))
        }
    }
}

pub async fn process(_rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    get_domain_record_key(params, RecordVersion::V2)
}

pub fn get_domain_record_key(
    params: Params,
    record_version: RecordVersion,
) -> Result<Value, crate::Error> {
    let record =
        Record::try_from_str(&params.record).map_err(|e| trace!(ErrorType::InvalidRecord, e))?;
    let domain_record_key = get_record_key(&params.domain, record, record_version)
        .map_err(|e| trace!(ErrorType::InvalidDomain, e))?;
    Ok(serde_json::to_value(domain_record_key.to_string())
        .map_err(|e| trace!(ErrorType::Generic, e)))?
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
            method: Method::GetDomainRecordKey,
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
            assert_eq!(value, "4sQDE98ZzQ23Rygb7tx1HhXQiuxswKhSBvECCREW35Ei");
        } else {
            let text = response.text().await.unwrap();
            eprintln!("Error body:\n {text}");
            panic!()
        }
    }

    #[tokio::test]
    async fn test_derivation() {
        struct Item {
            pub params: Params,
            pub version: RecordVersion,
            pub expected_result: String,
        }
        let items: Vec<Item> = vec![
            Item {
                params: Params {
                    domain: String::from("domain1"),
                    record: String::from("SOL"),
                },
                version: RecordVersion::V1,
                expected_result: String::from("ATH9akc5pi1PWDB39YY7VCoYzCxmz8XVj23oegSoNSPL"),
            },
            Item {
                params: Params {
                    domain: String::from("domain1"),
                    record: String::from("SOL"),
                },
                version: RecordVersion::V2,
                expected_result: String::from("GBrd6Q53eu1T2PiaQAtm92r3DwxmoGvZ2D6xjtVtN1Qt"),
            },
        ];
        for item in items.into_iter() {
            let res = get_domain_record_key(item.params, item.version).unwrap();
            let res = res.as_str().unwrap();
            assert_eq!(res, item.expected_result)
        }
    }
}
