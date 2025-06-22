use crate::models::*;
use rand::Rng;
use tokio::sync::mpsc;

/// Generates price ticks for two symbols whose returns share ρ ≈ 0.9.
pub struct Exchange {
    md_tx: mpsc::Sender<Tick>,
    fill_tx: mpsc::Sender<Fill>,
    px_a: f64,
    px_b: f64,
    symbol_a: String,
    symbol_b: String,
}

impl Exchange {
    pub fn new(md_tx: mpsc::Sender<Tick>,
               fill_tx: mpsc::Sender<Fill>,
               symbol_a: &str,
               symbol_b: &str,
               start_px: f64) -> Self
    {
        Self {
            md_tx,
            fill_tx,
            px_a: start_px,
            px_b: start_px * 0.98,
            symbol_a: symbol_a.to_string(),
            symbol_b: symbol_b.to_string(),
        }
    }

    pub async fn run(mut self, tick_ms: u64, mut od_rx: mpsc::Receiver<Order>) {
        let mut intv = tokio::time::interval(std::time::Duration::from_millis(tick_ms));
        let rho = 0.9;

        loop {
            tokio::select! {
                _ = intv.tick() => {
                    // correlated normal shocks
                    let n1 = rand::random::<f64>();
                    let n2 = rand::random::<f64>();
                    let z1 = (n1 - 0.5) * 0.002;                 // small drift
                    let z2 = rho*z1 + (1.0-rho.powi(2)).sqrt()*((n2-0.5)*0.002);

                    self.px_a *= (1.0 + z1);
                    self.px_b *= (1.0 + z2);

                    let now = std::time::Instant::now();
                    let _ = self.md_tx.send(Tick{symbol:self.symbol_a.clone(),px:self.px_a,ts:now}).await;
                    let _ = self.md_tx.send(Tick{symbol:self.symbol_b.clone(),px:self.px_b,ts:now}).await;
                }

                Some(ord) = od_rx.recv() => {
                    let book_px = if ord.symbol == self.symbol_a { self.px_a } else { self.px_b };
                    // cross at mid immediately
                    let fill = Fill { symbol: ord.symbol.clone(), px: book_px, qty: ord.qty, side: ord.side };
                    let _ = self.fill_tx.send(fill).await;
                }

                else => break,
            }
        }
    }
}
