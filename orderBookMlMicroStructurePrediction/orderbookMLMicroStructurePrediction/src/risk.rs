// MIT © 2025
use crate::{config::Cfg, models::*};
use std::time::Instant;

pub struct Risk {
    max_pos: f64,
    max_orders: usize,
    last_sec: Instant,
    sent_this_sec: usize,
    pos: f64,
}
impl Risk {
    pub fn new(cfg:&Cfg) -> Self {
        Self{ max_pos:cfg.max_pos, max_orders:cfg.max_orders_s,
              last_sec:Instant::now(), sent_this_sec:0, pos:0.0 }
    }
    pub fn allow(&mut self, o:&Order) -> bool {
        if self.last_sec.elapsed().as_secs() >= 1 {
            self.last_sec = Instant::now(); self.sent_this_sec = 0;
        }
        if self.sent_this_sec >= self.max_orders { return false; }
        let delta = if o.side==Side::Buy { o.qty } else { -o.qty };
        if (self.pos+delta).abs() > self.max_pos { return false; }
        self.pos += delta;
        self.sent_this_sec += 1;
        true
    }
}
