use ::config::{Config, ConfigBuilder, Environment, File, FileFormat};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Deserialize, Debug)]
pub(crate) struct ReputationServerConfig {
    pub listen_on: SocketAddr,
    pub apply_migrations: bool,
    pub database_url: String,
}

impl ReputationServerConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Config::builder()
            .set_default("listen_on", "127.0.0.1:8080")?
            .set_default("apply_migrations", true)?
            .add_source(Environment::with_prefix("repu"))
            .add_source(File::with_name("repu-config").required(false))
            .build()?
            .try_deserialize::<Self>()?)
    }
}
