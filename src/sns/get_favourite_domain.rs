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

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
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

    let favourite_domain_key = resolve::get_favourite_domain(&rpc_client, &owner)
        .await
        .map_err(|e| trace!(ErrorType::Generic, e))?;
    let result = if let Some(domain_key) = favourite_domain_key {
        let name = resolve::resolve_reverse(&rpc_client, &domain_key)
            .await
            .map_err(|e| trace!(ErrorType::Generic, e))?
            .ok_or(trace!(ErrorType::ReverseRecordNotFound))?;
        Some(ResultItem {
            name,
            key: domain_key.to_string(),
        })
    } else {
        None
    };
    Ok(serde_json::to_value(result).map_err(|e| trace!(ErrorType::Generic, e)))?
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn integrated_test_0() {
        use crate::sns::{Method, RpcMessage, RpcResponseOk, JSON_RPC};
        let endpoint = std::env::var("TEST_QUICKNODE_ENDPOINT").unwrap();
        let client = reqwest::Client::new();
        let message = RpcMessage {
            jsonrpc: JSON_RPC.to_owned(),
            method: Method::GetFavouriteDomain,
            params: serde_json::to_value(["HKKp49qGWXd639QsuH7JiLijfVW5UtCVY4s1n2HANwEA"]).unwrap(),
            id: serde_json::to_value(42u8).unwrap(),
        };
        eprintln!("{}", serde_json::to_string_pretty(&message).unwrap());
        let post_request = client.post(&endpoint).json(&message).build().unwrap();
        let response = client.execute(post_request).await.unwrap();
        eprintln!("{:#?}", response);
        if response.status().is_success() {
            let result: RpcResponseOk<String> = response.json().await.unwrap();
            let value = result.result.as_str().unwrap();
            assert_eq!(value, "Crf8hzfthWGbGbLTVCiqRqV5MVnbpHB1L9KQMd6gsinb");
        } else {
            let text = response.text().await.unwrap();
            eprintln!("Error body:\n {text}");
            panic!()
        }
    }
}
