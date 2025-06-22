//! Very simple position-limit checker.

use crate::{config::StratCfg, models::*};

pub struct Risk {
    cfg: StratCfg,
}

impl Risk {
    pub fn new(cfg: StratCfg) -> Self {
        Self { cfg }
    }

    /// Returns `true` if `pos_after` stays within ±pos_limit.
    pub fn allow(&self, _ord: &Order, pos_after: f64) -> bool {
        pos_after.abs() <= self.cfg.pos_limit
    }
}
