use std::sync::Arc;

use lazy_static::lazy_static;
use minimal_matrix::{mattermost::MatterMost, notif_trait::Notifier};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::config::CONFIG;
use std::ops::Deref;

#[derive(Clone, Default)]
pub struct MattermostClient {
    pub client: Option<minimal_matrix::mattermost::MatterMost>,
}

lazy_static! {
    pub static ref MATRIX_CLIENT: Arc<RwLock<Option<MattermostClient>>> =
        Arc::new(RwLock::new(None));
}

pub async fn init_matrix_client() {
    let mut m = MATRIX_CLIENT.write().await;
    *m = Some(MattermostClient::new().await);
}

pub fn get_matrix_client() -> impl Deref<Target = MattermostClient> {
    let m = MATRIX_CLIENT.try_read().unwrap();
    RwLockReadGuard::map(m, |r| r.as_ref().unwrap())
}

impl MattermostClient {
    pub async fn new() -> Self {
        let hook = CONFIG.mattermost_hook.as_deref();

        if let Some(hook) = hook {
            log::info!("Initializing matrix client");
            let client = MatterMost::new(hook);
            Self {
                client: Some(client),
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
