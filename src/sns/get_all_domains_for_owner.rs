use std::str::FromStr;

use crate::{append_trace, trace, ErrorType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sns_sdk::non_blocking::resolve;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use super::get_string_from_value_array;

#[derive(Deserialize)]
pub struct Params {
    owner: String,
}

#[derive(Serialize)]
pub struct ResultItem {
    name: String,
    key: String,
}

impl Params {
    pub fn deserialize(value: Value) -> Result<Self, crate::Error> {
        if let Some(v) = value.as_array() {
            let owner = get_string_from_value_array(v, 0).map_err(|e| append_trace!(e))?;
            Ok(Self { owner })
        } else {
            serde_json::from_value(value).map_err(|e| trace!(ErrorType::InvalidParameters, e))
        }
    }
}

pub async fn process(rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    let owner =
        Pubkey::from_str(&params.owner).map_err(|e| trace!(ErrorType::InvalidParameters, e))?;
    let domain_keys = resolve::get_domains_owner(&rpc_client, owner)
        .await
        .map_err(|e| trace!(ErrorType::Generic, e))?
        .into_iter()
        .collect::<Vec<_>>();
    let reversed = resolve::resolve_reverse_batch(&rpc_client, &domain_keys)
        .await
        .map_err(|e| trace!(ErrorType::Generic, e))?;
    let mut result = Vec::with_capacity(domain_keys.len());
    for (key, n) in domain_keys.into_iter().zip(reversed.into_iter()) {
        let name = n.ok_or(trace!(ErrorType::ReverseRecordNotFound))?;
        let key = key.to_string();
        result.push(ResultItem { name, key });
    }
    Ok(serde_json::to_value(result).map_err(|e| trace!(ErrorType::Generic, e)))?
}

#[cfg(test)]
#[tokio::test]
async fn integrated_test_0() {
    use crate::sns::{Method, RpcMessage, RpcResponseOk, JSON_RPC};
    let endpoint = std::env::var("TEST_QUICKNODE_ENDPOINT").unwrap();
    let client = reqwest::Client::new();
    let message = RpcMessage {
        jsonrpc: JSON_RPC.to_owned(),
        method: Method::GetAllDomainsForOwner,
        params: serde_json::to_value(["HKKp49qGWXd639QsuH7JiLijfVW5UtCVY4s1n2HANwEA"]).unwrap(),
        id: serde_json::to_value(42u8).unwrap(),
    };
    eprintln!("{}", serde_json::to_string_pretty(&message).unwrap());
    let post_request = client.post(&endpoint).json(&message).build().unwrap();
    let response = client.execute(post_request).await.unwrap();
    eprintln!("{:#?}", response);
    if response.status().is_success() {
        let result: RpcResponseOk<String> = response.json().await.unwrap();
        let value = result
            .result
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(value.len(), 4);
        assert!(value.contains("9B8y69VYEvLuwnaPdqNWL2wrV2XCLKrNAewC3FQEXptn"));
        assert!(value.contains("BAW7NsKcY8SLr98ZNYcH2HeDvPBPE2EoyjuPKcJ9bW1d"));
        assert!(value.contains("Crf8hzfthWGbGbLTVCiqRqV5MVnbpHB1L9KQMd6gsinb"));
        assert!(value.contains("8xMJaFHqas1gzS7xLuWh298TDuBUw4hqLXL2ZFs376hH"));
    } else {
        let text = response.text().await.unwrap();
        eprintln!("Error body:\n {text}");
        panic!()
    }
}
