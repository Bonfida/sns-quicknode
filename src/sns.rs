use std::fmt::Display;

use actix_web::{
    dev::HttpServiceFactory,
    http::header::{HeaderValue, CONTENT_TYPE},
    post, web, HttpRequest, ResponseError, Scope,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::nonblocking::rpc_client::RpcClient;

use crate::{db::DbConnector, trace, ErrorType};

pub mod resolve_domain;

pub fn scope() -> impl HttpServiceFactory {
    Scope::new("rpc").service(route)
}

pub enum Params<T> {
    Positional(Vec<String>),
    Named(T),
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Resolve,
    #[serde(other)]
    Unsupported,
}

#[derive(Deserialize)]
pub struct RpcMessage {
    pub jsonrpc: String,
    pub method: Method,
    pub params: Value,
    pub id: Value,
}

#[derive(Serialize)]
pub struct RpcResponseOk {
    pub jsonrpc: &'static str,
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
        self.0.fmt(f)
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
        let message = format!("{}", self.0);
        let body = RpcResponseError {
            error: RpcError {
                code: error_code as i64,
                message,
            },
            jsonrpc: JSON_RPC,
            id: self.0.clone(),
        };
        let mut res = actix_web::HttpResponse::new(self.status_code()).set_body(
            actix_web::body::BoxBody::new(serde_json::to_string(&body).unwrap_or_default()),
        );
        println!("Error : {self:?}");
        res.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        res
    }
}

#[post("/")]
pub async fn route(
    request: HttpRequest,
    message: web::Json<RpcMessage>,
    db: web::Data<DbConnector>,
) -> Result<web::Json<RpcResponseOk>, RpcErrorWrapper> {
    message.validate().map_err(|e| (message.id.clone(), e))?;
    let RpcMessage {
        params, id, method, ..
    } = message.into_inner();
    let rpc_client = get_rpc_client(&db, &request)
        .await
        .map_err(|e| (id.clone(), e))?;

    let result = match method {
        Method::Resolve => resolve_domain::run(
            rpc_client,
            resolve_domain::Params::deserialize(params).map_err(|e| (id.clone(), e))?,
        )
        .await
        .map_err(|e| (id.clone(), e))?,
        Method::Unsupported => {
            return Err((id.clone(), trace!(crate::ErrorType::UnsupportedEndpoint)).into())
        }
    };
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
        .get("X-QUICKNODE-ID")
        .ok_or(trace!(crate::ErrorType::InvalidAuthentication))?
        .to_str()
        .map_err(|e| trace!(crate::ErrorType::MalformedRequest, e))?;
    let provisioning_info = db.get_provisioning_request(quicknode_id).await?;
    let endpoint_url = provisioning_info.http_url;
    let rpc_client = RpcClient::new(endpoint_url);
    Ok(rpc_client)
}
