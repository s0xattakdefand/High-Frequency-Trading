use crate::{config::ArbCfg, models::*};
use ndarray::Array1;
use std::collections::BTreeMap;

pub struct BasisArb {
    cfg: ArbCfg,
    // rolling basis list (bps)
    basis_hist: Vec<f64>,
    // live positions
    pos_etf: f64,
    pos_stock: BTreeMap<String, f64>,
}

impl BasisArb {
    pub fn new(cfg: ArbCfg) -> Self {
        Self {
            pos_etf: 0.0,
            pos_stock: cfg
                .stock_syms()
                .into_iter()
                .map(|s| (s, 0.0))
                .collect(),
            basis_hist: Vec::with_capacity(cfg.lookback),
            cfg,
        }
    }

    pub fn pos_etf(&self) -> f64 { self.pos_etf }
    pub fn pos_stock(&self, sym:&str) -> f64 { self.pos_stock[sym] }

    /// Given latest prices, decide orders.
    pub fn on_tick(
        &mut self,
        etf_px: f64,
        stock_px: &BTreeMap<String, f64>,
    ) -> Vec<Order> {
        let fair = self
            .cfg
            .weights
            .iter()
            .map(|(s, w)| w * stock_px[s])
            .sum::<f64>();

        let basis_bps = (etf_px / fair - 1.0) * 10_000.0; // convert to bps

        // roll window
        if self.basis_hist.len() == self.cfg.lookback {
            self.basis_hist.remove(0);
        }
        self.basis_hist.push(basis_bps);

        if self.basis_hist.len() < self.cfg.lookback {
            return vec![];
        }

        // z-score of basis (mean≈0 by construction)
        let arr = Array1::from(self.basis_hist.clone());
        let std = arr.std(0.0).max(1e-4); // avoid div-by-0
        let z = basis_bps / std;

        let mut orders = Vec::new();

        // ---- entry signals ----
        if basis_bps > self.cfg.entry_bp
            && self.pos_etf - self.cfg.size_etf >= -self.cfg.pos_limit
        {
            // ETF rich → SELL ETF, BUY basket
            orders.push(Order {
                symbol: self.cfg.etf_sym().into(),
                px: etf_px,
                qty: self.cfg.size_etf,
                side: Side::Sell,
            });
            for (s, w) in &self.cfg.weights {
                orders.push(Order {
                    symbol: s.clone(),
                    px: stock_px[s],
                    qty: self.cfg.size_etf * w,
                    side: Side::Buy,
                });
            }
        } else if basis_bps < -self.cfg.entry_bp
            && self.pos_etf + self.cfg.size_etf <= self.cfg.pos_limit
        {
            // ETF cheap → BUY ETF, SELL basket
            orders.push(Order {
                symbol: self.cfg.etf_sym().into(),
                px: etf_px,
                qty: self.cfg.size_etf,
                side: Side::Buy,
            });
            for (s, w) in &self.cfg.weights {
                orders.push(Order {
                    symbol: s.clone(),
                    px: stock_px[s],
                    qty: self.cfg.size_etf * w,
                    side: Side::Sell,
                });
            }
        }

        // ---- flatten when inside exit band ----
        if basis_bps.abs() < self.cfg.exit_bp && self.pos_etf.abs() > 0.0 {
            // close ETF
            let side_etf = if self.pos_etf > 0.0 { Side::Sell } else { Side::Buy };
            orders.push(Order {
                symbol: self.cfg.etf_sym().into(),
                px: etf_px,
                qty: self.pos_etf.abs(),
                side: side_etf,
            });
            // close each stock
            for (s, p) in stock_px {
                let inv = self.pos_stock[s];
                if inv != 0.0 {
                    let side = if inv > 0.0 { Side::Sell } else { Side::Buy };
                    orders.push(Order {
                        symbol: s.clone(),
                        px: *p,
                        qty: inv.abs(),
                        side,
                    });
                }
            }
        }

        orders
    }

    /// Update position book.
    pub fn on_fill(&mut self, f: &Fill) {
        let delta = if matches!(f.side, Side::Buy) { f.qty } else { -f.qty };

        if f.symbol == self.cfg.etf_sym() {
            self.pos_etf += delta;
        } else {
            *self.pos_stock.get_mut(&f.symbol).unwrap() += delta;
        }
    }
}
