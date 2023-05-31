use crate::trace;
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
    Ok(serde_json::to_value(supported_records).map_err(|e| trace!(crate::ErrorType::Generic, e)))?
}
