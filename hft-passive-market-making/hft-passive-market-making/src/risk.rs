use crate::{models::*, config::MmCfg};

pub struct Risk {
    cfg: MmCfg,
    max_orders_sec: usize,
    sent_last_sec: usize,
    last_ts: std::time::Instant,
}

impl Risk {
    pub fn new(cfg: MmCfg) -> Self {
        Self { cfg, max_orders_sec: 20, sent_last_sec: 0,
               last_ts: std::time::Instant::now() }
    }

    pub fn allow(&mut self, _ord: &Order, inv_after: f64) -> bool {
        if inv_after.abs() > self.cfg.inv_limit { return false; }

        let now = std::time::Instant::now();
        if now.duration_since(self.last_ts).as_secs() >= 1 {
            self.sent_last_sec = 0;
            self.last_ts = now;
        }
        if self.sent_last_sec >= self.max_orders_sec { return false; }
        self.sent_last_sec += 1;
        true
    }
}
