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
    pub mattermost_hook: Option<String>,
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
            mattermost_hook: std::env::var("MATTERMOST_HOOK").ok(),
        }
    }
}
