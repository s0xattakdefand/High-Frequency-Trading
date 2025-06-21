//! Rolling-window Z-score pair-trading engine.

use crate::{config::StratCfg, models::*};
use ndarray::Array1;

pub struct PairTrader {
    cfg: StratCfg,
    // rolling log-price windows
    log_a: Vec<f64>,
    log_b: Vec<f64>,
    // live inventory per leg
    pos_a: f64,
    pos_b: f64,
}

/* ----- public helpers for outside code ----- */
impl PairTrader {
    pub fn pos_a(&self) -> f64 { self.pos_a }
    pub fn pos_b(&self) -> f64 { self.pos_b }
}

/* ----- core logic ----- */
impl PairTrader {
    pub fn new(cfg: StratCfg) -> Self {
        let look = cfg.lookback;
        Self {
            cfg,
            log_a: Vec::with_capacity(look),
            log_b: Vec::with_capacity(look),
            pos_a: 0.0,
            pos_b: 0.0,
        }
    }

    /// Consume latest prices, produce zero or more orders.
    pub fn on_ticks(&mut self, a_px: f64, b_px: f64) -> Vec<Order> {
        // update rolling window (pop oldest if full)
        if self.log_a.len() == self.cfg.lookback {
            self.log_a.remove(0);
            self.log_b.remove(0);
        }
        self.log_a.push(a_px.ln());
        self.log_b.push(b_px.ln());

        if self.log_a.len() < self.cfg.lookback {
            return vec![];
        }

        // compute current spread Z-score
        let la = Array1::from(self.log_a.clone());
        let lb = Array1::from(self.log_b.clone());
        let spread = &la - &(lb * self.cfg.beta);

        let mean = spread.mean().unwrap();
        let std  = spread.std(0.0).max(1e-8);
        let z    = (spread.last().unwrap() - mean) / std;

        let mut orders = Vec::new();

        // ----- entry logic -----
        if z > self.cfg.entry_z && self.pos_a - self.cfg.size >= -self.cfg.pos_limit {
            // short spread → sell A, buy B*β
            orders.push(Order {
                symbol: self.cfg.sym_a.clone(),
                px: a_px,
                qty: self.cfg.size,
                side: Side::Sell,
            });
            orders.push(Order {
                symbol: self.cfg.sym_b.clone(),
                px: b_px,
                qty: self.cfg.size * self.cfg.beta,
                side: Side::Buy,
            });
        } else if z < -self.cfg.entry_z && self.pos_a + self.cfg.size <= self.cfg.pos_limit {
            // long spread → buy A, sell B*β
            orders.push(Order {
                symbol: self.cfg.sym_a.clone(),
                px: a_px,
                qty: self.cfg.size,
                side: Side::Buy,
            });
            orders.push(Order {
                symbol: self.cfg.sym_b.clone(),
                px: b_px,
                qty: self.cfg.size * self.cfg.beta,
                side: Side::Sell,
            });
        }

        // ----- exit / flatten logic -----
        if z.abs() < self.cfg.exit_z && (self.pos_a != 0.0 || self.pos_b != 0.0) {
            let dir_a = if self.pos_a > 0.0 { Side::Sell } else { Side::Buy };
            let dir_b = if self.pos_b > 0.0 { Side::Sell } else { Side::Buy };
            orders.push(Order {
                symbol: self.cfg.sym_a.clone(),
                px: a_px,
                qty: self.pos_a.abs(),
                side: dir_a,
            });
            orders.push(Order {
                symbol: self.cfg.sym_b.clone(),
                px: b_px,
                qty: self.pos_b.abs(),
                side: dir_b,
            });
        }

        orders
    }

    /// Update internal inventory after a fill.
    pub fn on_fill(&mut self, f: &Fill) {
        let delta = if matches!(f.side, Side::Buy) { f.qty } else { -f.qty };
        if f.symbol == self.cfg.sym_a {
            self.pos_a += delta;
        } else if f.symbol == self.cfg.sym_b {
            self.pos_b += delta;
        }
    }
}
