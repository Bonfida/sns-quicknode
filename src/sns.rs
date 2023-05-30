use actix_web::{dev::HttpServiceFactory, HttpRequest, Scope};
use solana_client::nonblocking::rpc_client::RpcClient;

use crate::{db::DbConnector, trace};

pub mod resolve_domain;

pub fn scope() -> impl HttpServiceFactory {
    Scope::new("solana")
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
