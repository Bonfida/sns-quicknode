use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use actix_web::{
    http::header::{HeaderValue, CONTENT_TYPE},
    post, web, HttpRequest, ResponseError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::nonblocking::rpc_client::RpcClient;

use crate::{db::DbConnector, matrix::get_matrix_client, trace, ErrorType};

pub mod get_all_domains_for_owner;
pub mod get_domain_data;
pub mod get_domain_data_v2;
pub mod get_domain_key;
pub mod get_domain_record_key;
pub mod get_domain_record_v2_key;
pub mod get_domain_reverse_key;
pub mod get_favourite_domain;
pub mod get_registration_transaction;
pub mod get_subdomains;
pub mod get_supported_records;
pub mod resolve_domain;
pub mod reverse_lookup;

#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub enum Method {
    #[serde(rename = "sns_resolveDomain")]
    ResolveDomain,
    #[serde(rename = "sns_getDomainKey")]
    GetDomainKey,
    #[serde(rename = "sns_getAllDomainsForOwner")]
    GetAllDomainsForOwner,
    #[serde(rename = "sns_getDomainReverseKey")]
    GetDomainReverseKey,
    #[serde(rename = "sns_getDomainRecordKey")]
    GetDomainRecordKey,
    #[serde(rename = "sns_getDomainRecordV2Key")]
    GetDomainRecordV2Key,
    #[serde(rename = "sns_getFavouriteDomain")]
    GetFavouriteDomain,
    #[serde(rename = "sns_getSupportedRecords")]
    GetSupportedRecords,
    #[serde(rename = "sns_reverseLookup")]
    ReverseLookup,
    #[serde(rename = "sns_getSubdomains")]
    GetSubdomains,
    #[serde(rename = "sns_getRegistrationTransaction")]
    GetRegistrationTransaction,
    #[serde(rename = "sns_getDomainData")]
    GetDomainData,
    #[serde(rename = "sns_getDomainDataV2")]
    GetDomainDataV2,
    #[serde(other)]
    Unsupported,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub struct RpcMessage {
    pub jsonrpc: String,
    pub method: Method,
    pub params: Value,
    pub id: Value,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Deserialize))]
pub struct RpcResponseOk<T: Deref<Target = str>> {
    pub jsonrpc: T,
    pub result: Value,
    pub id: Value,
}

#[derive(Serialize)]
pub struct RpcResponseError {
    jsonrpc: &'static str,
    error: RpcError,
    id: Value,
}

#[derive(Serialize)]
pub struct RpcError {
    code: i64,
    message: String,
}

pub const JSON_RPC: &str = "2.0";

impl RpcMessage {
    pub fn validate(&self) -> Result<(), crate::Error> {
        if self.jsonrpc != "2.0" {
            return Err(trace!(crate::ErrorType::MalformedRequest));
        }
        Ok(())
    }
}

#[repr(i64)]
pub enum JsonRpcError {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerError = -32000,
}

#[derive(Debug)]
pub struct RpcErrorWrapper(Value, crate::Error);

impl Display for RpcErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Value as Display>::fmt(&self.0, f)
    }
}

impl From<(Value, crate::Error)> for RpcErrorWrapper {
    fn from(value: (Value, crate::Error)) -> Self {
        Self(value.0, value.1)
    }
}

impl ResponseError for RpcErrorWrapper {
    fn status_code(&self) -> actix_web::http::StatusCode {
        self.1.status_code()
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let error_code = match self.1.ty {
            ErrorType::InvalidAuthentication | ErrorType::ProvisioningRecordNotFound => {
                JsonRpcError::InvalidRequest
            }
            ErrorType::UnsupportedEndpoint => JsonRpcError::MethodNotFound,
            ErrorType::MalformedRequest | ErrorType::InvalidParameters => {
                JsonRpcError::InvalidParams
            }
            _ => JsonRpcError::ServerError,
        };
        let message = format!("{}", self.1);
        let body = RpcResponseError {
            error: RpcError {
                code: error_code as i64,
                message,
            },
            jsonrpc: JSON_RPC,
            id: self.0.clone(),
        };

        if !self.status_code().is_client_error() {
            let matrix_client = get_matrix_client();
            matrix_client.send_message(format!("Error: {self:#?}"));
        }
        let mut res = actix_web::HttpResponse::new(self.status_code()).set_body(
            actix_web::body::BoxBody::new(serde_json::to_string(&body).unwrap_or_default()),
        );
        log::error!("Error : {self:?}");
        res.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        res
    }
}

#[post("/rpc")]
pub async fn route(
    request: HttpRequest,
    message: web::Json<RpcMessage>,
    db: web::Data<DbConnector>,
) -> Result<web::Json<RpcResponseOk<&'static str>>, RpcErrorWrapper> {
    message.validate().map_err(|e| (message.id.clone(), e))?;

    let RpcMessage {
        params, id, method, ..
    } = message.into_inner();
    let rpc_client = get_rpc_client(&db, &request)
        .await
        .map_err(|e| (id.clone(), e))?;

    let result = match method {
        Method::ResolveDomain => resolve_domain::process(rpc_client, params).await,
        Method::GetDomainKey => get_domain_key::process(rpc_client, params).await,
        Method::GetAllDomainsForOwner => {
            get_all_domains_for_owner::process(rpc_client, params).await
        }
        Method::GetDomainReverseKey => get_domain_reverse_key::process(rpc_client, params).await,
        Method::GetDomainRecordKey => get_domain_record_key::process(rpc_client, params).await,
        Method::GetDomainRecordV2Key => get_domain_record_v2_key::process(rpc_client, params).await,
        Method::GetFavouriteDomain => get_favourite_domain::process(rpc_client, params).await,
        Method::GetSupportedRecords => get_supported_records::process(rpc_client, params).await,
        Method::ReverseLookup => reverse_lookup::process(rpc_client, params).await,
        Method::GetSubdomains => get_subdomains::process(rpc_client, params).await,
        Method::GetRegistrationTransaction => {
            get_registration_transaction::process(rpc_client, params).await
        }
        Method::GetDomainData => get_domain_data::process(rpc_client, params).await,
        Method::GetDomainDataV2 => get_domain_data_v2::process(rpc_client, params).await,
        Method::Unsupported => {
            return Err((id.clone(), trace!(crate::ErrorType::UnsupportedEndpoint)).into())
        }
    }
    .map_err(|e| (id.clone(), e))?;
    Ok(web::Json(RpcResponseOk {
        jsonrpc: JSON_RPC,
        result,
        id,
    }))
}

pub async fn get_rpc_client(
    db: &DbConnector,
    request: &HttpRequest,
) -> Result<RpcClient, crate::Error> {
    let quicknode_id = request
        .headers()
        .get("x-quicknode-id")
        .ok_or(trace!(crate::ErrorType::InvalidAuthentication))?
        .to_str()
        .map_err(|e| trace!(crate::ErrorType::MalformedRequest, e))?;
    let endpoint_id = request
        .headers()
        .get("x-instance-id")
        .ok_or(trace!(crate::ErrorType::InvalidAuthentication))?
        .to_str()
        .map_err(|e| trace!(crate::ErrorType::MalformedRequest, e))?;
    let provisioning_info = db
        .get_provisioning_request(quicknode_id, endpoint_id)
        .await?;
    let endpoint_url = provisioning_info.http_url;
    let rpc_client = RpcClient::new(endpoint_url);
    Ok(rpc_client)
}

fn get_string_from_value_array(array: &[Value], index: usize) -> Result<String, crate::Error> {
    let res = array
        .get(index)
        .ok_or(trace!(ErrorType::MissingParameters))?
        .as_str()
        .ok_or(trace!(ErrorType::InvalidParameters))?
        .to_owned();
    Ok(res)
}

fn get_opt_string_from_value_array(
    array: &[Value],
    index: usize,
) -> Result<Option<String>, crate::Error> {
    let res = array
        .get(index)
        .filter(|n| !n.is_null())
        .map(|v| v.as_str().ok_or(trace!(ErrorType::InvalidParameters)))
        .transpose()?
        .map(|v| v.to_owned());
    Ok(res)
}

fn get_int_from_value_array<T: TryFrom<u64>>(
    array: &[Value],
    index: usize,
) -> Result<T, crate::Error>
where
    <T as TryFrom<u64>>::Error: Debug,
{
    let res = array
        .get(index)
        .ok_or(trace!(ErrorType::MissingParameters))?
        .as_u64()
        .ok_or(trace!(ErrorType::InvalidParameters))?
        .try_into()
        .map_err(|e| trace!(ErrorType::InvalidParameters, e))?;
    Ok(res)
}

#[test]
pub fn method_name_deserialization_test() {
    let m = serde_json::to_string(&Method::ResolveDomain).unwrap();
    assert_eq!(m, "\"sns_resolveDomain\"");
}
