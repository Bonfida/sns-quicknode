use std::time::{SystemTime, UNIX_EPOCH};

use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use tokio_postgres::types::Type;

use crate::{config::CONFIG, provisioning::ProvisioningRequest, trace, ErrorType};

const DB_SCHEMA: &str = include_str!("sql/schema.sql");

pub struct DbConnector {
    pool: Pool,
}

impl DbConnector {
    pub async fn new() -> Self {
        let host = &CONFIG.postgres_host;
        let password = &CONFIG.postgres_password;
        let port = CONFIG.postgres_port;
        let mut config = tokio_postgres::Config::new();
        let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
        builder.set_ca_file("./certs/aws.pem").unwrap();
        let connector = MakeTlsConnector::new(builder.build());
        config
            .user("dbmasteruser")
            .dbname("postgres")
            .host(host)
            .password(password)
            .port(port);
        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(config, connector, mgr_config);
        let pool = Pool::builder(mgr).max_size(16).build().unwrap();

        Self { pool }
    }

    pub async fn init(&self) {
        self.pool
            .get()
            .await
            .unwrap()
            .query(DB_SCHEMA, &[])
            .await
            .unwrap();
    }

    pub async fn commit_provisioning_request(
        &self,
        request: &ProvisioningRequest,
    ) -> Result<(), crate::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| trace!(crate::ErrorType::DbError, e))?;
        let s = client
            .prepare_typed_cached(
                include_str!("sql/insert_provision.sql"),
                &[
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT_ARRAY,
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT,
                    Type::INT8,
                ],
            )
            .await
            .map_err(|e| trace!(ErrorType::DbError, e))?;
        client
            .execute(
                &s,
                &[
                    &request.quicknode_id,
                    &request.endpoint_id,
                    &request.wss_url,
                    &request.http_url,
                    &request.referers,
                    &request.chain,
                    &request.network,
                    &request.plan,
                    &i64::MAX,
                ],
            )
            .await
            .map_err(|e| trace!(ErrorType::DbError, e))?;
        Ok(())
    }

    pub async fn deprovision(
        &self,
        quicknode_id: &str,
        expiry_timestamp: i64,
    ) -> Result<(), crate::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| trace!(crate::ErrorType::DbError, e))?;
        let s = client
            .prepare_typed_cached(
                include_str!("sql/deprovision.sql"),
                &[Type::INT8, Type::TEXT],
            )
            .await
            .map_err(|e| trace!(ErrorType::DbError, e))?;
        client
            .execute(&s, &[&expiry_timestamp, &quicknode_id])
            .await
            .map_err(|e| trace!(ErrorType::DbError, e))?;
        Ok(())
    }

    pub async fn deactivate_endpoint(
        &self,
        quicknode_id: &str,
        endpoint_id: &str,
        expiry_timestamp: i64,
    ) -> Result<(), crate::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| trace!(crate::ErrorType::DbError, e))?;
        let s = client
            .prepare_typed_cached(
                include_str!("sql/deactivate_endpoint.sql"),
                &[Type::INT8, Type::TEXT, Type::TEXT],
            )
            .await
            .map_err(|e| trace!(ErrorType::DbError, e))?;
        client
            .execute(&s, &[&expiry_timestamp, &quicknode_id, &endpoint_id])
            .await
            .map_err(|e| trace!(ErrorType::DbError, e))?;
        Ok(())
    }

    pub async fn update_provisioning_request(
        &self,
        request: &ProvisioningRequest,
    ) -> Result<(), crate::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| trace!(crate::ErrorType::DbError, e))?;
        let s = client
            .prepare_typed_cached(
                include_str!("sql/update_provision.sql"),
                &[
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT_ARRAY,
                    Type::TEXT,
                    Type::TEXT,
                    Type::TEXT,
                    Type::INT8,
                    Type::TEXT,
                ],
            )
            .await
            .map_err(|e| trace!(ErrorType::DbError, e))?;
        client
            .execute(
                &s,
                &[
                    &request.endpoint_id,
                    &request.wss_url,
                    &request.http_url,
                    &request.referers,
                    &request.chain,
                    &request.network,
                    &request.plan,
                    &i64::MAX,
                    &request.quicknode_id,
                ],
            )
            .await
            .map_err(|e| trace!(ErrorType::DbError, e))?;
        Ok(())
    }

    pub async fn get_provisioning_request(
        &self,
        quicknode_id: &str,
        endpoint_id: &str,
    ) -> Result<ProvisioningRequest, crate::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| trace!(crate::ErrorType::DbError, e))?;
        let s = client
            .prepare_typed_cached(
                include_str!("sql/get_provision.sql"),
                &[Type::TEXT, Type::TEXT],
            )
            .await
            .map_err(|e| trace!(ErrorType::DbError, e))?;
        let record = client
            .query(&s, &[&quicknode_id, &endpoint_id])
            .await
            .map_err(|e| trace!(ErrorType::DbError, e))?
            .pop()
            .ok_or(trace!(ErrorType::ProvisioningRecordNotFound))?;
        let expiry_timestamp: i64 = record.get("expiry_timestamp");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        if expiry_timestamp <= now {
            Err(trace!(crate::ErrorType::ProvisioningRecordNotFound))
        } else {
            Ok(ProvisioningRequest {
                quicknode_id: record.get("quicknode_id"),
                endpoint_id: record.get("endpoint_id"),
                wss_url: record.get("wss_url"),
                http_url: record.get("http_url"),
                referers: record.get("referers"),
                chain: record.get("chain"),
                network: record.get("network"),
                plan: record.get("plan"),
            })
        }
    }
}
