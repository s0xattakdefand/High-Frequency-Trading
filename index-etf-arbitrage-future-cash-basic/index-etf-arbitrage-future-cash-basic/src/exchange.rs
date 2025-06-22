use crate::models::*;
use rand::Rng;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

/// Two-sided book simulator
pub struct Exchange {
    md_tx:   mpsc::Sender<Tick>,
    fill_tx: mpsc::Sender<Fill>,
    stocks:  BTreeMap<String, f64>, // live stock prices
    etf_px:  f64,                   // live ETF price
    weights: BTreeMap<String, f64>,
}

impl Exchange {
    pub fn new(
        md_tx: mpsc::Sender<Tick>,
        fill_tx: mpsc::Sender<Fill>,
        weights: &BTreeMap<String, f64>,
        start_px: f64,
    ) -> Self {
        let stocks = weights
            .keys()
            .map(|s| (s.clone(), start_px))
            .collect::<BTreeMap<_, _>>();
        Self { md_tx, fill_tx, stocks, etf_px: start_px, weights: weights.clone() }
    }

    fn fair_value(&self) -> f64 {
        self.weights
            .iter()
            .map(|(s, w)| w * self.stocks[s])
            .sum::<f64>()
    }

    pub async fn run(mut self, tick_ms: u64, mut od_rx: mpsc::Receiver<Order>) {
        let mut intv =
            tokio::time::interval(std::time::Duration::from_millis(tick_ms));
        loop {
            tokio::select! {
                _ = intv.tick() => {
                    // ---- random walk each stock ---
                    for px in self.stocks.values_mut() {
                        let noise = (rand::random::<f64>() - 0.5) * 0.002;
                        *px *= 1.0 + noise;
                    }

                    // ---- etf deviates from fair value by ±10 bps noise ----
                    let fv = self.fair_value();
                    let basis_noise = (rand::random::<f64>() - 0.5) * 0.001;
                    self.etf_px = fv * (1.0 + basis_noise);

                    let now = std::time::Instant::now();
                    // publish ticks
                    for (sym, px) in &self.stocks {
                        let _ = self.md_tx.send(Tick{symbol:sym.clone(),px:*px,ts:now}).await;
                    }
                    let _ = self.md_tx.send(Tick{symbol:"SIMETF".to_string(),px:self.etf_px,ts:now}).await;
                }

                Some(ord) = od_rx.recv() => {
                    // Fill at mid immediately
                    let px = if ord.symbol == "SIMETF" {
                        self.etf_px
                    } else {
                        self.stocks[&ord.symbol]
                    };
                    let fill = Fill{ symbol: ord.symbol.clone(), px, qty: ord.qty, side: ord.side };
                    let _ = self.fill_tx.send(fill).await;
                }

                else => break,
            }
        }
    }
}
