use crate::models::*;
use rand::Rng;
use tokio::sync::mpsc;

/// very small, in-process limit-order-book
pub struct Exchange {
    md_tx:   mpsc::Sender<Tick>,
    fill_tx: mpsc::Sender<Fill>,
    mid_px:  f64,
    spread:  f64,
}

impl Exchange {
    pub fn new(md_tx: mpsc::Sender<Tick>,
               fill_tx: mpsc::Sender<Fill>,
               start_mid: f64) -> Self
    {
        Self { md_tx, fill_tx, mid_px: start_mid, spread: 0.5 }
    }

    /// runs until *every* sender/receiver is dropped
    pub async fn run(mut self, tick_ms: u64, mut od_rx: mpsc::Receiver<Order>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(tick_ms));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // ---- random-walk the mid ----
                    let delta = {
                        let mut rng = rand::thread_rng();
                        rng.gen_range(-0.05..0.05)
                    };
                    self.mid_px += delta;

                    let tick = Tick {
                        bid: self.mid_px - self.spread,
                        ask: self.mid_px + self.spread,
                        ts:  std::time::Instant::now(),
                    };
                    if self.md_tx.send(tick).await.is_err() {
                        break;
                    }
                }

                Some(order) = od_rx.recv() => {
                    let price_ok = match order.side {
                        Side::Buy  => order.px >= self.mid_px + self.spread,
                        Side::Sell => order.px <= self.mid_px - self.spread,
                    };
                    if price_ok {
                        let fill = Fill { px: self.mid_px, qty: order.qty, side: order.side };
                        if self.fill_tx.send(fill).await.is_err() {
                            break;
                        }
                    }
                }

                else => break,
            }
        }
    }
}
