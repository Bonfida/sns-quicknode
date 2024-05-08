use serde_json::Value;
use sns_sdk::record::RecordVersion;
use solana_client::nonblocking::rpc_client::RpcClient;

use super::get_domain_data;

pub type Params = get_domain_data::Params;

pub async fn process(rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    get_domain_data::get_domain_data(rpc_client, params, RecordVersion::V2).await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn integrated_test_0() {
        use crate::sns::{Method, RpcMessage, RpcResponseOk, JSON_RPC};
        use base64::Engine;
        use solana_sdk::program_pack::Pack;
        use spl_name_service::state::NameRecordHeader;

        let endpoint = std::env::var("TEST_QUICKNODE_ENDPOINT").unwrap();
        let client = reqwest::Client::new();
        let message = RpcMessage {
            jsonrpc: JSON_RPC.to_owned(),
            method: Method::GetDomainDataV2,
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
