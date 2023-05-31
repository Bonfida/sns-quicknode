use actix_web::{dev::HttpServiceFactory, get, post, web, HttpRequest, Scope};
use serde::Deserialize;
use serde_json::Value;
use solana_client::nonblocking::rpc_client::RpcClient;

use crate::{db::DbConnector, trace};

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
    jsonrpc: String,
    method: Method,
    params: Value,
    id: Value,
}

impl RpcMessage {
    pub fn validate(&self) -> Result<(), crate::Error> {
        if self.jsonrpc != "2.0" {
            return Err(trace!(crate::ErrorType::MalformedRequest));
        }
        Ok(())
    }
}
pub async fn route(
    request: HttpRequest,
    message: web::Json<RpcMessage>,
    db: web::Data<DbConnector>,
) -> Result<String, crate::Error> {
    message.validate()?;
    let rpc_client = get_rpc_client(&db, &request).await?;
    let result = match message.method {
        Method::Resolve => resolve_domain::run(
            rpc_client,
            resolve_domain::Params::deserialize(message.params)?,
        ),
        Method::Unsupported => return Err(trace!(crate::ErrorType::UnsupportedEndpoint)),
    };
    let resolved = resolve::resolve_owner(&rpc_client, &domain)
        .await
        .map_err(|e| trace!(crate::ErrorType::Generic, e))?;
    Ok(resolved.to_string())
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
