use serde_json::Value;
use sns_sdk::record::RecordVersion;
use solana_client::nonblocking::rpc_client::RpcClient;

use super::get_domain_record_v2_key;

pub type Params = get_domain_record_v2_key::Params;

pub async fn process(_rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    get_domain_record_v2_key::get_domain_record_key(
        get_domain_record_v2_key::Params {
            domain: params.domain,
            record: params.record,
        },
        RecordVersion::V1,
    )
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
}
