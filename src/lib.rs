use actix_web::{get, web, App, HttpServer, Responder};
use actix_web_httpauth::extractors::basic::{self, BasicAuth};
use config::CONFIG;
use db::DbConnector;
pub use error::{Error, ErrorType};

use crate::matrix::{get_matrix_client, init_matrix_client, MattermostClient};

pub mod config;
pub mod db;
pub mod error;
pub mod matrix;
pub mod provisioning;
pub mod sns;

#[get("/hello")]
async fn greet(auth: BasicAuth) -> impl Responder {
    format!("Hello {}! ({:?})", auth.user_id(), auth.password())
}

#[get("/")]
async fn health() -> impl Responder {
    "ok"
}

pub async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    init_matrix_client().await;
    pretty_env_logger::init_timed();

    let matrix_client = get_matrix_client();
    log::info!("Launching server");

    let db = web::Data::new(DbConnector::new().await);

    db.init().await;
    log::info!("Connected to db");

    matrix_client.send_message("Server instance successfully initialized".to_owned());

    HttpServer::new(move || {
        let authentication_config = basic::Config::default().realm("Restricted API");
        App::new()
            .app_data(authentication_config)
            .app_data(web::Data::clone(&db))
            .wrap(actix_web::middleware::Logger::default())
            .service(greet)
            .service(health)
            .service(provisioning::scope())
            .service(sns::route)
    })
    .bind(("0.0.0.0", CONFIG.port))?
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

pub fn log_matrix<C: AsRef<MattermostClient>>(matrix_client: Option<C>, msg: String) {
    log::error!("{}", msg);
    if let Some(c) = matrix_client.as_ref() {
        c.as_ref().send_message(msg);
    }
}
