// MIT © 2025
//! Synthetic 5-level order-book stream & naive fill engine.

use crate::{config::Cfg, models::*};
use rand::random;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

pub async fn run(
    cfg: Cfg,
    book_tx: mpsc::Sender<Book>,
    mut ord_rx: mpsc::Receiver<Order>,
    fill_tx: mpsc::Sender<Fill>,
) {
    let mut mid = 100.00;
    let mut clock = interval(Duration::from_millis(cfg.tick_ms));

    loop {
        tokio::select! {
            /* -------- 1-ms synthetic book update -------- */
            _ = clock.tick() => {
                // random-walk mid-price
                mid += if random::<f64>() > 0.5 { cfg.tick_sz } else { -cfg.tick_sz };

                // random depths for five price levels
                let mut bid = [0.0; 5];
                let mut ask = [0.0; 5];
                for i in 0..5 {
                    bid[i] = 500.0 + random::<f64>() * 1000.0;
                    ask[i] = 500.0 + random::<f64>() * 1000.0;
                }

                let book = Book { bid_vol: bid, ask_vol: ask, mid };
                let _ = book_tx.send(book).await;          // await after RNG is dropped
            }

            /* -------- order from ML strategy -------- */
            Some(o) = ord_rx.recv() => {
                let px  = if o.side == Side::Buy { mid + cfg.tick_sz }
                                           else { mid - cfg.tick_sz };
                let fill = Fill {
                    px,
                    qty:  o.qty,
                    side: o.side,
                    ts:   tokio::time::Instant::now().into_std(),
                };
                let _ = fill_tx.send(fill).await;
            }

            else => break,
        }
    }
}
