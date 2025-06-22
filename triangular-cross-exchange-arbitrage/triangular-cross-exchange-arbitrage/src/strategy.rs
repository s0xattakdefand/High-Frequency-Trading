//! EUR → USD → JPY → EUR triangular-arbitrage engine.

use crate::{config::TriCfg, models::*};
use std::collections::{BTreeMap, HashMap};

pub struct TriArb {
    cfg: TriCfg,
    px:  BTreeMap<String, (f64, f64)>,   // pair → (bid, ask)
    pos: HashMap<&'static str, f64>,     // currency → inventory
}

impl TriArb {
    pub fn new(cfg: TriCfg) -> Self {
        Self {
            cfg,
            px:  BTreeMap::new(),
            pos: [("EUR", 0.0), ("USD", 0.0), ("JPY", 0.0)].into(),
        }
    }

    /* ────────── market-data ────────── */
    pub fn update_tick(&mut self, t: Tick) { self.px.insert(t.pair, (t.bid, t.ask)); }

    /* ────────── signal logic ───────── */
    pub fn check(&self) -> Vec<Order> {
        if self.px.len() < 3 { return vec![]; }

        let (eu_b, eu_a) = self.px["EUR/USD"];
        let (uj_b, uj_a) = self.px["USD/JPY"];
        let (ej_b, ej_a) = self.px["EUR/JPY"];

        let fee = self.cfg.fee_bps / 10_000.0;
        let qty = self.cfg.size_eur;

        // path-1 EUR→USD→JPY→EUR
        let eur_out1 = qty * eu_b * (1.0 - fee)
                           * uj_b * (1.0 - fee)
                       / ej_a / (1.0 + fee);

        // path-2 EUR→JPY→USD→EUR
        let eur_out2 = qty * ej_b * (1.0 - fee)
                       / uj_a / (1.0 + fee)
                       / eu_a / (1.0 + fee);

        let edge1 = (eur_out1 / qty - 1.0) * 10_000.0;
        let edge2 = (eur_out2 / qty - 1.0) * 10_000.0;

        let mut orders = Vec::new();

        if edge1 > self.cfg.entry_bps {
            orders.push(Order { pair:"EUR/USD".into(), side:Side::Sell, qty_base:qty });
            orders.push(Order { pair:"USD/JPY".into(), side:Side::Buy,  qty_base:qty * eu_b });
            orders.push(Order { pair:"EUR/JPY".into(), side:Side::Buy,  qty_base:qty });
        } else if edge2 > self.cfg.entry_bps {
            orders.push(Order { pair:"EUR/JPY".into(), side:Side::Sell, qty_base:qty });
            orders.push(Order { pair:"USD/JPY".into(), side:Side::Sell,
                                qty_base:qty * ej_b / eu_a });
            orders.push(Order { pair:"EUR/USD".into(), side:Side::Buy,  qty_base:qty });
        }
        orders
    }

    /* ────────── fill processing ─────── */
    pub fn on_fill(&mut self, f: &Fill) {
        match f.pair.as_str() {
            /* EUR/USD -------------------------------------------------- */
            "EUR/USD" => {
                let usd = f.qty_base * f.price;
                if matches!(f.side, Side::Buy) {
                    *self.pos.get_mut("EUR").unwrap() += f.qty_base;
                    *self.pos.get_mut("USD").unwrap() -= usd;
                } else {
                    *self.pos.get_mut("EUR").unwrap() -= f.qty_base;
                    *self.pos.get_mut("USD").unwrap() += usd;
                }
            }
            /* USD/JPY -------------------------------------------------- */
            "USD/JPY" => {
                let jpy = f.qty_base * f.price;
                if matches!(f.side, Side::Buy) {
                    *self.pos.get_mut("USD").unwrap() -= f.qty_base;
                    *self.pos.get_mut("JPY").unwrap() += jpy;
                } else {
                    *self.pos.get_mut("USD").unwrap() += f.qty_base;
                    *self.pos.get_mut("JPY").unwrap() -= jpy;
                }
            }
            /* EUR/JPY -------------------------------------------------- */
            "EUR/JPY" => {
                let jpy = f.qty_base * f.price;
                if matches!(f.side, Side::Buy) {
                    *self.pos.get_mut("EUR").unwrap() += f.qty_base;
                    *self.pos.get_mut("JPY").unwrap() -= jpy;
                } else {
                    *self.pos.get_mut("EUR").unwrap() -= f.qty_base;
                    *self.pos.get_mut("JPY").unwrap() += jpy;
                }
            }
            _ => {}
        }
    }

    /* ── inventory after hypothetical exec (risk check helper) ── */
    pub fn pos_after_exec(&self, ord: &Order) -> HashMap<&'static str, f64> {
        let mut p = self.pos.clone();
        let bid = self.px[&ord.pair].0;

        match ord.pair.as_str() {
            "EUR/USD" => {
                let usd = ord.qty_base * bid;
                if matches!(ord.side, Side::Buy) {
                    *p.get_mut("EUR").unwrap() += ord.qty_base;
                    *p.get_mut("USD").unwrap() -= usd;
                } else {
                    *p.get_mut("EUR").unwrap() -= ord.qty_base;
                    *p.get_mut("USD").unwrap() += usd;
                }
            }
            "USD/JPY" => {
                let jpy = ord.qty_base * bid;
                if matches!(ord.side, Side::Buy) {
                    *p.get_mut("USD").unwrap() -= ord.qty_base;
                    *p.get_mut("JPY").unwrap() += jpy;
                } else {
                    *p.get_mut("USD").unwrap() += ord.qty_base;
                    *p.get_mut("JPY").unwrap() -= jpy;
                }
            }
            "EUR/JPY" => {
                let jpy = ord.qty_base * bid;
                if matches!(ord.side, Side::Buy) {
                    *p.get_mut("EUR").unwrap() += ord.qty_base;
                    *p.get_mut("JPY").unwrap() -= jpy;
                } else {
                    *p.get_mut("EUR").unwrap() -= ord.qty_base;
                    *p.get_mut("JPY").unwrap() += jpy;
                }
            }
            _ => {}
        }
        p
    }
}
