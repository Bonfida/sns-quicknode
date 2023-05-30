use crate::{db::DbConnector, sns::get_rpc_client, trace};
use actix_web::{get, web, HttpRequest};
use sns_sdk::non_blocking::resolve;

#[get("/resolve/{domain}")]
pub async fn route(
    request: HttpRequest,
    domain: web::Path<String>,
    db: web::Data<DbConnector>,
) -> Result<String, crate::Error> {
    let rpc_client = get_rpc_client(&db, &request).await?;
    let resolved = resolve::resolve_owner(&rpc_client, &domain)
        .await
        .map_err(|e| trace!(crate::ErrorType::Generic, e))?;
    Ok(resolved.to_string())
}
