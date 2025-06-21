//! Loads strategy parameters from `Config.toml` or `STB_*` env vars.

use anyhow::Result;
use figment::providers::{Env, Format, Toml};
use serde::Deserialize;

/// All configurable knobs for the stat-arb pair trader.
#[derive(Debug, Deserialize, Clone)]
pub struct StratCfg {
    /// First leg of the pair (e.g. “AAPL”)
    pub sym_a: String,
    /// Second leg of the pair (e.g. “MSFT”)
    pub sym_b: String,

    /// Rolling-window length in ticks.
    pub lookback: usize,

    /// β coefficient in   spread = log P_A − β·log P_B
    pub beta: f64,

    /// Z-score at which to open a spread trade.
    pub entry_z: f64,
    /// Z-score band inside which we flatten.
    pub exit_z: f64,

    /// Contract size per leg.
    pub size: f64,

    /// Hard cap on absolute position per leg.
    pub pos_limit: f64,

    /// Simulator tick interval in milliseconds.
    pub tick_ms: u64,
}

impl StratCfg {
    /// Read `Config.toml`, then override with `STB_*` environment variables.
    pub fn load() -> Result<Self> {
        Ok(figment::Figment::from(Toml::file("Config.toml"))
            .merge(Env::prefixed("STB_"))
            .extract()?)
    }
}
