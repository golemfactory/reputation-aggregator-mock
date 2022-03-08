use std::net::SocketAddr;
use serde::Deserialize;
use ::config::{Config, Environment, ConfigBuilder, File, FileFormat};

#[derive(Deserialize, Debug)]
pub(crate) struct ReputationServerConfig {
    pub api_listen : SocketAddr,
    pub database_url : String
}

impl ReputationServerConfig {

    pub fn load() -> anyhow::Result<Self> {
        Ok(Config::builder()
            .set_default("api_listen", "127.0.0.1:8080")?
            .add_source(Environment::with_prefix("repu"))
            .add_source(File::with_name("config").required(false))
            .build()?.try_deserialize::<Self>()?)
    }
}