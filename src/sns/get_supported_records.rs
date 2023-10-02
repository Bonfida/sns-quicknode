use crate::{trace, ErrorType};
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::record::Record;
use solana_client::nonblocking::rpc_client::RpcClient;

#[derive(Deserialize)]
pub struct Params {}

impl Params {
    pub fn deserialize(_value: Value) -> Result<Self, crate::Error> {
        Ok(Self {})
    }
}

pub async fn process(_rpc_client: RpcClient, _params: Value) -> Result<Value, crate::Error> {
    let supported_records = [
        Record::Ipfs.as_str(),
        Record::Arwv.as_str(),
        Record::Sol.as_str(),
        Record::Eth.as_str(),
        Record::Btc.as_str(),
        Record::Ltc.as_str(),
        Record::Doge.as_str(),
        Record::Email.as_str(),
        Record::Url.as_str(),
        Record::Discord.as_str(),
        Record::Github.as_str(),
        Record::Reddit.as_str(),
        Record::Twitter.as_str(),
        Record::Telegram.as_str(),
        Record::Pic.as_str(),
        Record::Shdw.as_str(),
        Record::Point.as_str(),
        Record::Bsc.as_str(),
        Record::Injective.as_str(),
        Record::Backpack.as_str(),
    ];
    Ok(serde_json::to_value(supported_records).map_err(|e| trace!(ErrorType::Generic, e)))?
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
            method: Method::GetSupportedRecords,
            params: serde_json::to_value(["bonfida.sol"]).unwrap(),
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
            assert_eq!(value.len(), 20);
            eprintln!("{value:?}");
        } else {
            let text = response.text().await.unwrap();
            eprintln!("Error body:\n {text}");
            panic!()
        }
    }
}
