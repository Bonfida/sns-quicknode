use std::str::FromStr;

use crate::{append_trace, trace, ErrorType};
use base64::Engine;
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::non_blocking::register::register_domain_name;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use super::{
    get_int_from_value_array, get_opt_string_from_value_array, get_string_from_value_array,
};

#[derive(Deserialize)]
pub struct Params {
    domain: String,
    buyer: String,
    buyer_token_account: String,
    space: u32,
    mint: Option<String>,
    referrer_key: Option<String>,
}

impl Params {
    pub fn deserialize(value: Value) -> Result<Self, crate::Error> {
        if let Some(v) = value.as_array() {
            let domain = get_string_from_value_array(v, 0).map_err(|e| append_trace!(e))?;
            let buyer = get_string_from_value_array(v, 1).map_err(|e| append_trace!(e))?;
            let buyer_token_account =
                get_string_from_value_array(v, 2).map_err(|e| append_trace!(e))?;
            let space = get_int_from_value_array(v, 3).map_err(|e| append_trace!(e))?;
            let mint = get_opt_string_from_value_array(v, 4).map_err(|e| append_trace!(e))?;
            let referrer_key =
                get_opt_string_from_value_array(v, 5).map_err(|e| append_trace!(e))?;
            Ok(Self {
                domain,
                buyer,
                buyer_token_account,
                space,
                mint,
                referrer_key,
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
        buyer,
        buyer_token_account,
        space,
        mint,
        referrer_key,
    } = params;
    let buyer = Pubkey::from_str(&buyer).map_err(|e| trace!(ErrorType::InvalidParameters, e))?;
    let buyer_token_account = Pubkey::from_str(&buyer_token_account)
        .map_err(|e| trace!(ErrorType::InvalidParameters, e))?;

    let mint = mint
        .map(|k| Pubkey::from_str(&k))
        .transpose()
        .map_err(|e| trace!(ErrorType::InvalidParameters, e))?;
    let referrer_key = referrer_key
        .map(|k| Pubkey::from_str(&k))
        .transpose()
        .map_err(|e| trace!(ErrorType::InvalidParameters, e))?;
    let register_transaction = register_domain_name(
        rpc_client,
        &domain,
        space,
        &buyer,
        &buyer_token_account,
        mint.as_ref(),
        referrer_key.as_ref(),
    )
    .await
    .map_err(|e| trace!((&e).into(), e))?;
    let serialized_transaction =
        bincode::serialize(&register_transaction).map_err(|e| trace!(ErrorType::Generic, e))?;
    let encoded_transaction =
        base64::engine::general_purpose::STANDARD.encode(serialized_transaction);
    serde_json::to_value(encoded_transaction).map_err(|e| trace!(ErrorType::Generic, e))
}
