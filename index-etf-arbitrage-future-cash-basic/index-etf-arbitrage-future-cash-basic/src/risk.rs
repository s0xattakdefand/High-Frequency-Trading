use crate::{config::ArbCfg, models::*};

pub struct Risk { cfg: ArbCfg }

impl Risk {
    pub fn new(cfg: ArbCfg) -> Self { Self { cfg } }

    pub fn allow(&self, sym:&str, inv_after:f64) -> bool {
        inv_after.abs() <= self.cfg.pos_limit
            && (sym == "SIMETF" || self.cfg.weights.contains_key(sym))
    }
}
