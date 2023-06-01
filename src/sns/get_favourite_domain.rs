use std::str::FromStr;

use crate::{append_trace, trace, ErrorType};
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::non_blocking::resolve::get_favourite_domain;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use super::get_string_from_value_array;

#[derive(Deserialize)]
pub struct Params {
    owner: String,
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

    let favourite_domain = get_favourite_domain(&rpc_client, &owner)
        .await
        .map_err(|e| trace!(ErrorType::Generic, e))?
        .as_ref()
        .map(<Pubkey as ToString>::to_string);
    Ok(serde_json::to_value(favourite_domain).map_err(|e| trace!(ErrorType::Generic, e)))?
}
