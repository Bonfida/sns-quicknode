use actix_web::{get, post, put, web, App, HttpResponse, HttpServer, Responder, ResponseError};
use actix_web_httpauth::extractors::basic::{self, BasicAuth};
use base64::Engine;
use config::CONFIG;
use db::DbConnector;
pub use error::{Error, ErrorType};
use serde::{Deserialize, Serialize};

pub mod config;
pub mod db;
pub mod error;
pub mod provisioning;

#[get("/hello")]
async fn greet(auth: BasicAuth) -> impl Responder {
    format!("Hello {}! ({:?})", auth.user_id(), auth.password())
}

#[get("/")]
async fn health() -> impl Responder {
    "ok"
}

pub async fn main() -> std::io::Result<()> {
    println!("Launching server");
    let db = web::Data::new(DbConnector::new().await);
    println!("Connected to db");
    db.init().await;
    let credential_string = format!(
        "{}:{}",
        CONFIG.quicknode_username, CONFIG.quicknode_password
    );
    let encoded_credentials = base64::engine::general_purpose::STANDARD.encode(&credential_string);
    println!("{encoded_credentials}");
    HttpServer::new(move || {
        let authentication_config = basic::Config::default().realm("Restricted API");
        App::new()
            .app_data(authentication_config)
            .app_data(web::Data::clone(&db))
            .service(greet)
            .service(health)
            .service(provisioning::scope())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

pub fn validate_basic_auth(auth: BasicAuth) -> Result<(), crate::Error> {
    if auth.user_id() != CONFIG.quicknode_username
        || auth.password() != Some(&CONFIG.quicknode_password)
    {
        Err(trace!(crate::ErrorType::InvalidAuthentication))
    } else {
        Ok(())
    }
}
