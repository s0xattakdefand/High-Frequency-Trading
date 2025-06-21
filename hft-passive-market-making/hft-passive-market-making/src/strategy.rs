use crate::{config::MmCfg, models::*};

pub struct InventoryMm {
    cfg: MmCfg,
    inv: f64,
    pnl: f64,
}

impl InventoryMm {
    pub fn new(cfg: MmCfg) -> Self {
        Self { cfg, inv: 0.0, pnl: 0.0 }
    }

    pub fn quote(&self, tick: &Tick) -> (f64, f64) {
        let mid = (tick.bid + tick.ask) / 2.0;
        let skew = (self.inv / self.cfg.inv_limit).clamp(-1.0, 1.0);
        let half = self.cfg.half_spread * (1.0 + skew.abs() * self.cfg.inv_spread_mult);
        (mid - half, mid + half)
    }

    pub fn on_fill(&mut self, f: &Fill) {
        let dir = if matches!(f.side, Side::Buy) { 1.0 } else { -1.0 };
        self.inv += dir * f.qty;
        self.pnl -= dir * f.qty * f.px;
    }

    pub fn inv(&self) -> f64 { self.inv }
    pub fn pnl(&self) -> f64 { self.pnl }
}
