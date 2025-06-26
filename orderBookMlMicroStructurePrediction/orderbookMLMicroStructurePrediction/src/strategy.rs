// MIT © 2025
//! Online logistic regression using 5-level depth imbalance as features.

use crate::{config::Cfg, models::*};

pub struct MLTrader {
    cfg: Cfg,
    w:   [f64; 5],      // weights for 5 features
    b:   f64,           // bias
    last_feat: [f64; 5],
    last_mid:  f64,
    have_prev: bool,
}

impl MLTrader {
    pub fn new(cfg: Cfg) -> Self {
        Self {
            cfg,
            w: [0.0; 5],
            b: 0.0,
            last_feat: [0.0; 5],
            last_mid: 0.0,
            have_prev: false,
        }
    }

    /// Called on every new book snapshot.
    pub fn on_book(&mut self, book: &Book) -> Option<Order> {
        /* -------- build feature vector -------- */
        let mut x = [0.0; 5];
        for i in 0..5 {
            let num = book.bid_vol[i] - book.ask_vol[i];
            let den = book.bid_vol[i] + book.ask_vol[i];
            x[i] = if den > 0.0 { num / den } else { 0.0 };  // imbalance ∈ [-1,1]
        }

        /* -------- forward pass -------- */
        let z: f64 = (0..5).map(|i| self.w[i] * x[i]).sum::<f64>() + self.b;
        let p_up = sigmoid(z);

        /* -------- online SGD (learn from previous tick) -------- */
        if self.have_prev {
            let y = if book.mid > self.last_mid { 1.0 } else { 0.0 };      // label
            let z_prev: f64 = (0..5).map(|i| self.w[i] * self.last_feat[i]).sum::<f64>() + self.b;
            let pred = sigmoid(z_prev);
            let err  = y - pred;                                           // (y-ŷ)

            for i in 0..5 {
                self.w[i] += self.cfg.learning_rate * err * self.last_feat[i];
            }
            self.b += self.cfg.learning_rate * err;
        }
        self.last_feat = x;
        self.last_mid  = book.mid;
        self.have_prev = true;

        /* -------- trade when confident -------- */
        if  p_up - 0.5 >  self.cfg.theta {
            Some(Order { side: Side::Buy,  qty: 1000.0 })
        } else if 0.5 - p_up > self.cfg.theta {
            Some(Order { side: Side::Sell, qty: 1000.0 })
        } else {
            None
        }
    }

    pub fn on_fill(&mut self, _f: &Fill) {}
}

#[inline]
fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}
