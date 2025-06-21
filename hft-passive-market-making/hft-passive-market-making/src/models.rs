#[derive(Debug, Clone)]
pub struct Tick {
    pub bid: f64,
    pub ask: f64,
    pub ts:  std::time::Instant,
}

#[derive(Debug, Clone)]
pub enum Side { Buy, Sell }

#[derive(Debug, Clone)]
pub struct Order {
    pub px:   f64,
    pub qty:  f64,
    pub side: Side,
}

#[derive(Debug, Clone)]
pub struct Fill {
    pub px:   f64,
    pub qty:  f64,
    pub side: Side,
}
