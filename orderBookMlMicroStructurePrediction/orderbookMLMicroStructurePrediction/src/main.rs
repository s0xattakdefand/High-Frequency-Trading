// MIT © 2025
mod config;
mod models;
mod simulator;
mod strategy;
mod risk;

use anyhow::Result;
use tracing::info;
use tokio::sync::mpsc;
use models::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = config::Cfg::load()?;

    /* channels */
    let (book_tx, mut books) = mpsc::channel::<Book>(1024);
    let (ord_tx,  mut ord_rx)= mpsc::channel::<Order>(1024);
    let (fill_tx, mut fills) = mpsc::channel::<Fill>(1024);

    tokio::spawn(simulator::run(cfg.clone(), book_tx, ord_rx, fill_tx));

    let mut strat = strategy::MLTrader::new(cfg.clone());
    let mut risk  = risk::Risk::new(&cfg);

    loop {
        tokio::select! {
            Some(book) = books.recv() => {
                if let Some(o) = strat.on_book(&book) {
                    if risk.allow(&o) { ord_tx.send(o).await?; }
                }
            }
            Some(f) = fills.recv() => {
                strat.on_fill(&f);
                info!("FILL {:?} {:.0} @ {:.2}", f.side, f.qty, f.px);
            }
            else => break,
        }
    }
    Ok(())
}
