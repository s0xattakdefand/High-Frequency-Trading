mod config;
mod models;
mod exchange;
mod strategy;
mod risk;

use anyhow::Result;
use tracing::info;
use tokio::sync::mpsc;

use crate::config::StratCfg;
use crate::models::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = StratCfg::load()?;

    // ---------- channels ----------
    let (md_tx, mut md_rx)   = mpsc::channel::<Tick>(2048);
    let (fill_tx, mut fills) = mpsc::channel::<Fill>(2048);
    let (od_tx, od_rx)       = mpsc::channel::<Order>(2048);

    // ---------- start simulator ----------
    tokio::spawn(
        exchange::Exchange::new(md_tx, fill_tx, &cfg.sym_a, &cfg.sym_b, 100.0)
            .run(cfg.tick_ms, od_rx),
    );

    // ---------- strategy + risk ----------
    let mut strat   = strategy::PairTrader::new(cfg.clone());
    let     riskmgr = risk::Risk::new(cfg.clone());

    // cache latest prices so we call on_ticks() once per interval
    let mut last_a = 100.0;
    let mut last_b = 98.0;

    loop {
        tokio::select! {
            /* ---------- market-data ---------- */
            Some(tick) = md_rx.recv() => {
                if tick.symbol == cfg.sym_a { last_a = tick.px; }
                else                        { last_b = tick.px; }

                // call strategy only when we have both legs (B arrives last)
                if tick.symbol == cfg.sym_b {
                    let orders = strat.on_ticks(last_a, last_b);

                    for o in orders {
                        let delta = if matches!(o.side, Side::Buy) { o.qty } else { -o.qty };
                        let pos_after = if o.symbol == cfg.sym_a {
                            strat.pos_a() + delta
                        } else {
                            strat.pos_b() + delta
                        };

                        if riskmgr.allow(&o, pos_after) {
                            od_tx.send(o).await?;
                        }
                    }
                }
            }

            /* ---------- fills ---------- */
            Some(fill) = fills.recv() => {
                strat.on_fill(&fill);
                info!("FILL {:?} {:.2} {} @ {:.2}",
                      fill.side, fill.qty, fill.symbol, fill.px);
            }

            else => break,
        }
    }

    Ok(())
}
