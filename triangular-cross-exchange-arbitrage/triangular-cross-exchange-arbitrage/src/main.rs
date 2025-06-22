mod config;
mod models;
mod exchange;
mod strategy;
mod risk;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

use crate::{config::TriCfg, models::*};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = TriCfg::load()?;

    // channels
    let (md_tx, mut md_rx)   = mpsc::channel::<Tick>(2048);
    let (fill_tx, mut fills) = mpsc::channel::<Fill>(2048);
    let (od_tx,  od_rx)      = mpsc::channel::<Order>(2048);

    // spawn exchange
    tokio::spawn(
        exchange::Exchange::new(md_tx, fill_tx, &cfg.spreads, 1.10, 150.0)
            .run(cfg.tick_ms, od_rx)
    );

    // strategy + risk
    let mut strat  = strategy::TriArb::new(cfg.clone());
    let riskmgr    = risk::Risk::new(cfg.clone());

    loop {
        tokio::select! {
            Some(t) = md_rx.recv() => {
                strat.update_tick(t);
                let orders = strat.check();

                for o in orders {
                    let pos_after = strat.pos_after_exec(&o);
                    if riskmgr.allow(&pos_after) {
                        od_tx.send(o).await?;
                    }
                }
            }

            Some(f) = fills.recv() => {
                strat.on_fill(&f);
                info!("FILL {:?} {:.0} {} at {:.4}", f.side, f.qty_base, f.pair, f.price);
            }

            else => break,
        }
    }

    Ok(())
}
