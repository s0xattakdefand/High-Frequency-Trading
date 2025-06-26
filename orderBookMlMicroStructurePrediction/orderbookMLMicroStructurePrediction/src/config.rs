// MIT © 2025
use anyhow::Result;
use figment::providers::{Env, Format, Toml};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Cfg {
    pub symbol:       String,
    pub tick_sz:      f64,
    pub tick_ms:      u64,

    pub theta:        f64,
    pub learning_rate:f64,

    pub max_pos:      f64,
    pub max_orders_s: usize,
}

impl Cfg {
    pub fn load() -> Result<Self> {
        Ok(figment::Figment::from(Toml::file("Config.toml"))
            .merge(Env::prefixed("MLLOB_"))
            .extract()?)
    }
}
