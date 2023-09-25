use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::config::CONFIG;
use std::ops::Deref;

#[derive(Clone, Default)]
pub struct MatrixClient {
    pub client: Option<minimal_matrix::matrix::MatrixClient>,
}

lazy_static! {
    pub static ref MATRIX_CLIENT: Arc<RwLock<Option<MatrixClient>>> = Arc::new(RwLock::new(None));
}

pub async fn init_matrix_client() {
    let mut m = MATRIX_CLIENT.write().await;
    *m = Some(MatrixClient::new().await);
}

pub fn get_matrix_client() -> impl Deref<Target = MatrixClient> {
    let m = MATRIX_CLIENT.try_read().unwrap();
    RwLockReadGuard::map(m, |r| r.as_ref().unwrap())
}

impl MatrixClient {
    pub async fn new() -> Self {
        let home_server_name = CONFIG.home_server_name.clone();
        let room_id = CONFIG.room_id.clone();
        let access_token = CONFIG.access_token.clone();

        if let (Some(home_server_name), Some(room_id), Some(access_token)) =
            (home_server_name, room_id, access_token)
        {
            log::info!("Initializing matrix client");
            let client =
                minimal_matrix::matrix::MatrixClient::new(home_server_name, room_id, access_token)
                    .await;
            match client {
                Ok(client) => Self {
                    client: Some(client),
                },
                Err(err) => {
                    log::error!("Error logging into matrix - {}", err);
                    Default::default()
                }
            }
        } else {
            log::info!("No matrix client");
            Default::default()
        }
    }
    pub fn send_message(&self, message: String) {
        if let Some(c) = &self.client {
            match c.send_message(message) {
                Ok(_) => (),
                Err(err) => log::error!("Failed to send Matrix message: {}", err),
            }
        }
    }
}
