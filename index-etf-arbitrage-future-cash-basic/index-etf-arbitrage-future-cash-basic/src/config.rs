//! Load strategy parameters.

use anyhow::Result;
use figment::providers::{Env, Format, Toml};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ArbCfg {
    /// Basket weights — map symbol → weight.
    pub weights: BTreeMap<String, f64>,

    pub lookback: usize,
    pub entry_bp: f64,
    pub exit_bp:  f64,
    pub size_etf: f64,
    pub pos_limit: f64,
    pub tick_ms: u64,
}

impl ArbCfg {
    pub fn load() -> Result<Self> {
        Ok(figment::Figment::from(Toml::file("Config.toml"))
            .merge(Env::prefixed("IDX_"))
            .extract()?)
    }

    /// Convenience helpers
    pub fn etf_sym(&self) -> &'static str { "SIMETF" }
    pub fn stock_syms(&self) -> Vec<String> {
        self.weights.keys().cloned().collect()
    }
}
