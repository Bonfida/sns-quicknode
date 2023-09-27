use std::fmt::Display;

use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError, Scope};
use actix_web_httpauth::extractors::basic::BasicAuth;
use serde::{Deserialize, Serialize};

use crate::{db::DbConnector, matrix::get_matrix_client, validate_basic_auth};

#[get("/test/{quicknode_id}")]
async fn test(
    basic_auth: BasicAuth,
    quicknode_id: web::Path<String>,
    db: web::Data<DbConnector>,
) -> impl Responder {
    validate_basic_auth(basic_auth)?;
    let record = db.get_provisioning_request(&quicknode_id).await?;
    Result::<_, crate::Error>::Ok(web::Json(record))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProvisioningRequest {
    pub quicknode_id: String,
    pub endpoint_id: String,
    pub wss_url: String,
    pub http_url: String,
    pub referers: Vec<String>,
    pub chain: String,
    pub network: String,
    pub plan: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProvisioningDeactivateRequest {
    pub quicknode_id: String,
    pub endpoint_id: String,
    pub deactivate_at: i64,
    pub chain: String,
    pub network: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeprovisioningRequest {
    pub quicknode_id: String,
    pub endpoint_id: String,
    pub deprovision_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResponseStatus {
    Success,
    Error,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProvisioniningResponse {
    pub status: ResponseStatus,
    pub dashboard_url: Option<String>,
    pub access_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProvisioniningUpdateResponse {
    status: ResponseStatus,
}

#[derive(Debug)]
pub struct ProvisioningError(crate::Error);

impl Display for ProvisioningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ResponseError for ProvisioningError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        self.0.status_code()
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let status_code = self.status_code();
        if status_code.is_server_error() {
            log::error!("Server Error: {:#?}", self.0);
        }
        if !self.status_code().is_client_error() {
            let matrix_client = get_matrix_client();
            matrix_client.send_message(format!("Error: {self:#?}"));
        }
        HttpResponse::build(self.status_code()).json(ProvisioniningUpdateResponse {
            status: ResponseStatus::Error,
        })
    }
}

impl From<crate::Error> for ProvisioningError {
    fn from(value: crate::Error) -> Self {
        Self(value)
    }
}

#[post("/provision")]
async fn new(
    basic_auth: BasicAuth,
    request: web::Json<ProvisioningRequest>,
    db: web::Data<DbConnector>,
) -> Result<web::Json<ProvisioniningResponse>, ProvisioningError> {
    validate_basic_auth(basic_auth)?;
    db.commit_provisioning_request(&request).await?;
    Ok(web::Json(ProvisioniningResponse {
        status: ResponseStatus::Success,
        dashboard_url: None,
        access_url: None,
    }))
}

#[put("/update")]
async fn update(
    basic_auth: BasicAuth,
    request: web::Json<ProvisioningRequest>,
    db: web::Data<DbConnector>,
) -> Result<web::Json<ProvisioniningUpdateResponse>, ProvisioningError> {
    validate_basic_auth(basic_auth)?;
    db.update_provisioning_request(&request).await?;
    Ok(web::Json(ProvisioniningUpdateResponse {
        status: ResponseStatus::Success,
    }))
}

#[delete("/deactivate_endpoint")]
async fn deactivate(
    basic_auth: BasicAuth,
    request: web::Json<ProvisioningDeactivateRequest>,
    db: web::Data<DbConnector>,
) -> Result<web::Json<ProvisioniningUpdateResponse>, ProvisioningError> {
    validate_basic_auth(basic_auth)?;

    let deactivate_at = request.deactivate_at;

    db.deactivate_endpoint(&request.quicknode_id, &request.endpoint_id, deactivate_at)
        .await?;
    Ok(web::Json(ProvisioniningUpdateResponse {
        status: ResponseStatus::Success,
    }))
}

#[delete("/deprovision")]
async fn deprovision(
    basic_auth: BasicAuth,
    request: web::Json<DeprovisioningRequest>,
    db: web::Data<DbConnector>,
) -> Result<web::Json<ProvisioniningUpdateResponse>, ProvisioningError> {
    validate_basic_auth(basic_auth)?;
    let deprovision_at = request.deprovision_at;
    db.deprovision(&request.quicknode_id, deprovision_at)
        .await?;
    Ok(web::Json(ProvisioniningUpdateResponse {
        status: ResponseStatus::Success,
    }))
}

pub fn scope() -> Scope {
    Scope::new("provisioning")
        .service(test)
        .service(new)
        .service(update)
        .service(deactivate)
        .service(deprovision)
}
