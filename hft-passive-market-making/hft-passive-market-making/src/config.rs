use serde::Deserialize;
use figment::providers::{Env, Toml, Format};  // ← `Format` gives Toml::file()

#[derive(Debug, Deserialize, Clone)]
pub struct MmCfg {
    /// symbol traded in the sim
    pub symbol: String,
    /// ticks half-spread when flat
    pub half_spread: f64,
    /// contracts per quote side
    pub size: f64,
    /// inventory hard-limit
    pub inv_limit: f64,
    /// extra spread at max inventory (multiplier)
    pub inv_spread_mult: f64,
    /// simulator tick interval (ms)
    pub tick_ms: u64,
}

impl MmCfg {
    pub fn load() -> anyhow::Result<Self> {
        Ok(figment::Figment::from(Toml::file("Config.toml"))
            .merge(Env::prefixed("MM_"))
            .extract()?)
    }
}
