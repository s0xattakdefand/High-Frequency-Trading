//! Per-currency notional cap.

use crate::config::TriCfg;
use std::collections::HashMap;

pub struct Risk {
    cfg: TriCfg,
}

impl Risk {
    pub fn new(cfg: TriCfg) -> Self { Self { cfg } }

    /// Are all currency inventories within ±pos_limit?
    pub fn allow(&self, pos_after: &HashMap<&'static str, f64>) -> bool {
        pos_after.values().all(|p| p.abs() <= self.cfg.pos_limit)
    }
}
