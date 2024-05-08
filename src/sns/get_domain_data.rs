use crate::{append_trace, trace, ErrorType};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sns_records::state::{
    record_header::RecordHeader,
    validation::{get_validation_length, Validation},
};
use sns_sdk::{
    derivation::get_domain_key,
    record::{Record, RecordVersion},
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_name_service::state::NameRecordHeader;

use super::{get_opt_string_from_value_array, get_string_from_value_array};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    domain: String,
    record: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum QueryResult {
    V1(Option<String>),
    V2 {
        current_owner: String,
        content: String,
        staleness_id: String,
        staleness_validation: String,
        roa_id: String,
        roa_validation: String,
    },
}

impl Params {
    pub fn deserialize(value: Value) -> Result<Self, crate::Error> {
        if let Some(v) = value.as_array() {
            let domain = get_string_from_value_array(v, 0).map_err(|e| append_trace!(e))?;
            let record = get_opt_string_from_value_array(v, 1).map_err(|e| append_trace!(e))?;
            Ok(Self { domain, record })
        } else {
            serde_json::from_value(value).map_err(|e| trace!(ErrorType::InvalidParameters, e))
        }
    }
}

pub async fn process(rpc_client: RpcClient, params: Value) -> Result<Value, crate::Error> {
    let params = Params::deserialize(params)?;
    get_domain_data(rpc_client, params, RecordVersion::V1).await
}

pub async fn get_domain_data(
    rpc_client: RpcClient,
    params: Params,
    record_version: RecordVersion,
) -> Result<Value, crate::Error> {
    let Params { domain, record } = params;
    let record = record
        .map(|s| Record::try_from_str(&s))
        .transpose()
        .map_err(|e| trace!(ErrorType::InvalidRecord, e))?
        .map(|r| sns_sdk::record::get_record_key(&domain, r, record_version).map(|k| (r, k)))
        .transpose()
        .map_err(|e| trace!(ErrorType::InvalidDomain, e))?;
    let result = match (record, record_version) {
        (None, _) | (_, RecordVersion::V1) => {
            let account_key = Ok(record.map(|d| d.1)).transpose().unwrap_or_else(|| {
                get_domain_key(&domain).map_err(|e| trace!(ErrorType::InvalidDomain, e))
            })?;
            let account = rpc_client
                .get_account_with_commitment(&account_key, rpc_client.commitment())
                .await
                .map_err(|e| trace!(ErrorType::SolanaRpcError, e))?
                .value;
            let data = account.map(|a| {
                base64::engine::general_purpose::STANDARD.encode(&a.data[NameRecordHeader::LEN..])
            });
            QueryResult::V1(data)
        }
        (Some((record, record_key)), RecordVersion::V2) => {
            let domain_key =
                get_domain_key(&domain).map_err(|e| trace!(ErrorType::InvalidDomain, e))?;
            let accounts = rpc_client
                .get_multiple_accounts(&[domain_key, record_key])
                .await
                .map_err(|e| trace!(ErrorType::SolanaRpcError, e))?;
            let domain_account = accounts
                .first()
                .ok_or(trace!(ErrorType::Generic))?
                .as_ref()
                .ok_or(trace!(ErrorType::InvalidDomain))?;
            let record_account =
                if let Some(r) = accounts.get(1).ok_or(trace!(ErrorType::Generic))? {
                    r
                } else {
                    return serde_json::to_value(Option::<QueryResult>::None)
                        .map_err(|e| trace!(ErrorType::Generic, e));
                };
            let domain_header = NameRecordHeader::unpack_unchecked(&domain_account.data)
                .map_err(|e| trace!(ErrorType::Generic, e))?;
            if record_account.data.len() < NameRecordHeader::LEN + RecordHeader::LEN {
                return Err(trace!(ErrorType::InvalidRecord));
            }
            let record_v2_header = sns_records::state::record_header::RecordHeader::from_buffer(
                &record_account.data[NameRecordHeader::LEN..],
            );
            let roa_validation =
                Validation::try_from(record_v2_header.right_of_association_validation)
                    .map_err(|e| trace!(ErrorType::InvalidRecord, e))?;
            let roa_len = get_validation_length(roa_validation) as usize;
            let staleness_validation = Validation::try_from(record_v2_header.staleness_validation)
                .map_err(|e| trace!(ErrorType::InvalidRecord, e))?;
            let staleness_len = get_validation_length(staleness_validation) as usize;
            let staleness_id_offset = NameRecordHeader::LEN + RecordHeader::LEN;
            let roa_offset = staleness_id_offset + staleness_len;
            let content_offset = roa_offset + roa_len;
            if record_account.data.len()
                < content_offset + (record_v2_header.content_length as usize)
            {
                return Err(trace!(ErrorType::InvalidRecord));
            }
            let staleness_id = parse_validation_id(
                &record_account.data[staleness_id_offset..],
                staleness_validation,
            )?;
            let roa_id = parse_validation_id(&record_account.data[roa_offset..], roa_validation)?;
            let data = sns_sdk::record::record_v2::deserialize_record_v2_content(
                &domain_account.data[content_offset..],
                record,
            )
            .map_err(|e| trace!(ErrorType::InvalidRecord, e))?;
            QueryResult::V2 {
                current_owner: domain_header.owner.to_string(),
                content: data,
                staleness_id,
                staleness_validation: parse_validation(&staleness_validation).to_owned(),
                roa_id,
                roa_validation: parse_validation(&roa_validation).to_owned(),
            }
        }
    };
    serde_json::to_value(result).map_err(|e| trace!(ErrorType::Generic, e))
}

fn parse_validation_id(buffer: &[u8], validation: Validation) -> Result<String, crate::Error> {
    let result = match validation {
        Validation::None | Validation::UnverifiedSolana => "".to_owned(),
        Validation::Solana => Pubkey::try_from(&buffer[..32])
            .map_err(|e| trace!(ErrorType::InvalidRecord, e))?
            .to_string(),
        Validation::Ethereum => format!("0x{}", base16::encode_lower(&buffer[..20])),
        Validation::XChain => format!(
            "{{\"chainId\":{},\"ownerKey\":{}}}",
            u16::from_le_bytes(buffer[..2].try_into().unwrap()),
            serde_json::to_string(&buffer[2..34]).map_err(|e| trace!(ErrorType::Generic, e))?
        ),
    };
    Ok(result)
}

fn parse_validation(validation: &Validation) -> &'static str {
    match validation {
        Validation::None | Validation::UnverifiedSolana => "None",
        Validation::Solana => "Solana",
        Validation::Ethereum => "Ethereum",
        Validation::XChain => "XChain",
    }
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
            method: Method::GetDomainData,
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
