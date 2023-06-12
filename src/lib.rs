use actix_web::{get, web, App, HttpServer, Responder};
use actix_web_httpauth::extractors::basic::{self, BasicAuth};
use config::CONFIG;
use db::DbConnector;
pub use error::{Error, ErrorType};

use crate::matrix::{get_matrix_client, init_matrix_client, MatrixClient};

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
    init_matrix_client().await;
    let matrix_client = get_matrix_client();
    println!("Launching server");
    let db = web::Data::new(DbConnector::new().await);
    db.init().await;
    println!("Connected to db");
    matrix_client.send_message("Server instance successfully initialized".to_owned());
    // let credential_string = format!(
    //     "{}:{}",
    //     CONFIG.quicknode_username, CONFIG.quicknode_password
    // );
    // let encoded_credentials = base64::engine::general_purpose::STANDARD.encode(&credential_string);
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // println!("{encoded_credentials}");
    HttpServer::new(move || {
        let authentication_config = basic::Config::default().realm("Restricted API");
        App::new()
            .app_data(authentication_config)
            .app_data(web::Data::clone(&db))
            .service(greet)
            .service(health)
            .service(provisioning::scope())
            .service(sns::route)
        // .wrap_fn(move |req, srv| {
        //     let is_health_checker = req
        //         .headers()
        //         .get("user-agent")
        //         .and_then(|v| v.to_str().ok())
        //         .map(|s| s == "ELB-HealthChecker/2.0")
        //         .unwrap_or(false);
        //     if !is_health_checker {
        //         m_c.send_message(format!("{req:?}"));
        //     }
        //     srv.call(req)
        // })
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

pub fn log_matrix<C: AsRef<MatrixClient>>(matrix_client: Option<C>, msg: String) {
    eprintln!("{}", msg);
    if let Some(c) = matrix_client.as_ref() {
        c.as_ref().send_message(msg);
    }
}
