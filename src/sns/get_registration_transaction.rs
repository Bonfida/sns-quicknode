use std::str::FromStr;

use crate::{trace, ErrorType};
use base64::Engine;
use serde::Deserialize;
use serde_json::Value;
use sns_sdk::non_blocking::register::register_domain_name;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

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
            let domain = v
                .get(0)
                .ok_or(trace!(ErrorType::MissingParameters))?
                .as_str()
                .ok_or(trace!(ErrorType::InvalidParameters))?
                .to_owned();
            let buyer = v
                .get(1)
                .ok_or(trace!(ErrorType::MissingParameters))?
                .as_str()
                .ok_or(trace!(ErrorType::InvalidParameters))?
                .to_owned();
            let buyer_token_account = v
                .get(2)
                .ok_or(trace!(ErrorType::MissingParameters))?
                .as_str()
                .ok_or(trace!(ErrorType::InvalidParameters))?
                .to_owned();
            let space = v
                .get(3)
                .ok_or(trace!(ErrorType::MissingParameters))?
                .as_u64()
                .ok_or(trace!(ErrorType::InvalidParameters))?
                .to_owned()
                .try_into()
                .map_err(|e| trace!(ErrorType::InvalidParameters, e))?;
            let mint = v
                .get(4)
                .filter(|n| !n.is_null())
                .map(|v| {
                    v.as_str()
                        .map(|v| v.to_owned())
                        .ok_or(trace!(ErrorType::InvalidParameters))
                })
                .transpose()?;
            let referrer_key = v
                .get(5)
                .filter(|n| !n.is_null())
                .map(|v| {
                    v.as_str()
                        .map(|v| v.to_owned())
                        .ok_or(trace!(ErrorType::InvalidParameters))
                })
                .transpose()?;
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
