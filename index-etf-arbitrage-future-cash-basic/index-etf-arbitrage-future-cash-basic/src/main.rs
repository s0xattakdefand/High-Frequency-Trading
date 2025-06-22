mod config;
mod models;
mod exchange;
mod strategy;
mod risk;

use anyhow::Result;
use tracing::info;
use tokio::sync::mpsc;
use std::collections::BTreeMap;

use crate::{config::ArbCfg, models::*};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = ArbCfg::load()?;

    // channels
    let (md_tx, mut md_rx)   = mpsc::channel::<Tick>(4096);
    let (fill_tx, mut fills) = mpsc::channel::<Fill>(4096);
    let (od_tx, od_rx)       = mpsc::channel::<Order>(4096);

    // start sim
    tokio::spawn(
        exchange::Exchange::new(md_tx, fill_tx, &cfg.weights, 100.0)
            .run(cfg.tick_ms, od_rx),
    );

    // strategy + risk
    let mut strat = strategy::BasisArb::new(cfg.clone());
    let riskmgr   = risk::Risk::new(cfg.clone());

    // latest price cache
    let mut px_map: BTreeMap<String, f64> = cfg
        .weights
        .keys()
        .cloned()
        .map(|s| (s, 100.0))
        .collect();
    px_map.insert(cfg.etf_sym().into(), 100.0);

    loop {
        tokio::select! {
            Some(t) = md_rx.recv() => {
                px_map.insert(t.symbol.clone(), t.px);

                // invoke strategy when ETF tick arrives (ensures all stocks updated first)
                if t.symbol == cfg.etf_sym() {
                    let orders = strat.on_tick(t.px, &px_map);
                    for o in orders {
                        // inventory after hypothetical fill
                        let inv_after = if o.symbol == cfg.etf_sym() {
                            strat.pos_etf() + if matches!(o.side,Side::Buy){o.qty}else{-o.qty}
                        } else {
                            strat.pos_stock(&o.symbol) + if matches!(o.side,Side::Buy){o.qty}else{-o.qty}
                        };

                        if riskmgr.allow(&o.symbol, inv_after) {
                            od_tx.send(o).await?;
                        }
                    }
                }
            }

            Some(f) = fills.recv() => {
                strat.on_fill(&f);
                info!("FILL {:?} {:.0} {} @ {:.2}", f.side, f.qty, f.symbol, f.px);
            }

            else => break,
        }
    }

    Ok(())
}
