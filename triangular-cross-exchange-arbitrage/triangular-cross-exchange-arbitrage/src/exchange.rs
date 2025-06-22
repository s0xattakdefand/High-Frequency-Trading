//! One-venue simulator for EUR/USD, USD/JPY and EUR/JPY.

use crate::models::*;
use rand::random;                               // only `random()` is used
use std::collections::BTreeMap;
use tokio::sync::mpsc;

pub struct Exchange {
    md_tx:   mpsc::Sender<Tick>,
    fill_tx: mpsc::Sender<Fill>,
    mid:     BTreeMap<String, f64>,              // pair → mid-price
    spread:  BTreeMap<String, f64>,              // pair → fixed spread
}

impl Exchange {
    pub fn new(
        md_tx: mpsc::Sender<Tick>,
        fill_tx: mpsc::Sender<Fill>,
        spreads: &BTreeMap<String, f64>,
        start_eur_usd: f64,
        start_usd_jpy: f64,
    ) -> Self {
        let mut mid = BTreeMap::new();
        mid.insert("EUR/USD".into(), start_eur_usd);
        mid.insert("USD/JPY".into(), start_usd_jpy);
        mid.insert("EUR/JPY".into(), start_eur_usd * start_usd_jpy);

        Self { md_tx, fill_tx, mid, spread: spreads.clone() }
    }

    pub async fn run(mut self, tick_ms: u64, mut od_rx: mpsc::Receiver<Order>) {
        let mut intv = tokio::time::interval(std::time::Duration::from_millis(tick_ms));

        loop {
            tokio::select! {
                /* ---- publish ticks ---- */
                _ = intv.tick() => {
                    // tiny random walk on the two legs
                    for &(pair, vol) in &[("EUR/USD", 0.00005), ("USD/JPY", 0.005)] {
                        let bump = (random::<f64>() - 0.5) * vol;
                        *self.mid.get_mut(pair).unwrap() *= 1.0 + bump;
                    }
                    // create small mis-pricing on EUR/JPY
                    let off = (random::<f64>() - 0.5) * 0.002;
                    self.mid.insert(
                        "EUR/JPY".into(),
                        self.mid["EUR/USD"] * self.mid["USD/JPY"] * (1.0 + off),
                    );

                    let now = std::time::Instant::now();
                    for (pair, mid) in &self.mid {
                        let spr = self.spread[pair];
                        let _ = self.md_tx.send(Tick {
                            pair: pair.clone(),
                            bid:  mid - spr / 2.0,
                            ask:  mid + spr / 2.0,
                            ts:   now,
                        }).await;
                    }
                }

                /* ---- fill market orders immediately at touch ---- */
                Some(o) = od_rx.recv() => {
                    let spr   = self.spread[&o.pair];
                    let mid   = self.mid[&o.pair];
                    let price = if matches!(o.side, Side::Buy) { mid + spr/2.0 }
                                else                            { mid - spr/2.0 };

                    let _ = self.fill_tx.send(Fill {
                        pair:     o.pair.clone(),
                        side:     o.side.clone(),
                        qty_base: o.qty_base,
                        price,
                    }).await;
                }

                else => break,
            }
        }
    }
}
