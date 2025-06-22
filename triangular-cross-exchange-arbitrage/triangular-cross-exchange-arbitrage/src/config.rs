//! Load user knobs.

use anyhow::Result;
use figment::providers::{Env, Format, Toml};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Clone)]
pub struct TriCfg {
    pub entry_bps:  f64,
    pub exit_bps:   f64,
    pub fee_bps:    f64,
    pub size_eur:   f64,
    pub pos_limit:  f64,
    pub tick_ms:    u64,
    pub spreads:    BTreeMap<String, f64>,
}

impl TriCfg {
    pub fn load() -> Result<Self> {
        Ok(figment::Figment::from(Toml::file("Config.toml"))
            .merge(Env::prefixed("TRI_"))
            .extract()?)
    }
}
