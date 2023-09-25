lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::from_env();
}

pub struct Config {
    pub quicknode_username: String,
    pub quicknode_password: String,
    pub port: u16,
    pub postgres_host: String,
    pub postgres_password: String,
    pub postgres_port: u16,
    pub home_server_name: Option<String>,
    pub room_id: Option<String>,
    pub access_token: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            quicknode_username: std::env::var("QUICKNODE_USERNAME").unwrap(),
            quicknode_password: std::env::var("QUICKNODE_PASSWORD").unwrap(),
            postgres_host: std::env::var("POSTGRES_HOST").unwrap(),
            postgres_password: std::env::var("POSTGRES_PASSWORD").unwrap(),
            postgres_port: std::env::var("POSTGRES_PORT").unwrap().parse().unwrap(),
            port: std::env::var("BIND_PORT").unwrap().parse().unwrap(),
            home_server_name: std::env::var("MATRIX_HOME_SERVER_NAME").ok(),
            room_id: std::env::var("MATRIX_ROOM_ID").ok(),
            access_token: std::env::var("MATRIX_ACCESS_TOKEN").ok(),
        }
    }
}
