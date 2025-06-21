mod config;
mod models;
mod exchange;
mod strategy;
mod risk;

use tracing::info;
use tokio::sync::mpsc;
use crate::models::*;
use crate::config::MmCfg;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = MmCfg::load()?;

    // -------- channels --------
    let (md_tx, mut md_rx)   = mpsc::channel::<Tick>(1024);
    let (fill_tx, mut fills) = mpsc::channel::<Fill>(1024);
    let (od_tx, od_rx)       = mpsc::channel::<Order>(1024);

    // -------- spawn exchange sim --------
    let ex  = exchange::Exchange::new(md_tx, fill_tx, 100.0);
    tokio::spawn(ex.run(cfg.tick_ms, od_rx));

    // -------- trading state --------
    let mut mm   = strategy::InventoryMm::new(cfg.clone());
    let mut risk = risk::Risk::new(cfg.clone());

    // -------- event loop --------
    loop {
        tokio::select! {
            Some(tick) = md_rx.recv() => {
                let (bid, ask) = mm.quote(&tick);

                for (px, side) in [(bid, Side::Buy), (ask, Side::Sell)] {
                    let qty = cfg.size;
                    let dir = if matches!(side, Side::Buy) { 1.0 } else { -1.0 };
                    let inv_after = mm.inv() + dir * qty;
                    let ord = Order { px, qty, side: side.clone() };

                    if risk.allow(&ord, inv_after) {
                        od_tx.send(ord).await?;
                    }
                }
            }

            Some(fill) = fills.recv() => {
                mm.on_fill(&fill);
                info!("FILL {:?} qty={:.0} px={:.2} inv={:.1} pnl={:.2}",
                      fill.side, fill.qty, fill.px, mm.inv(), mm.pnl());
            }

            else => break,
        }
    }
    Ok(())
}
