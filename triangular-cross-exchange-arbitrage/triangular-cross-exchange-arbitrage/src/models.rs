#[derive(Clone, Debug)]
pub struct Tick {
    pub pair: String,     // e.g. "EUR/USD"
    pub bid:  f64,
    pub ask:  f64,
    pub ts:   std::time::Instant,
}

#[derive(Clone, Debug)]
pub enum Side { Buy, Sell }

#[derive(Clone, Debug)]
pub struct Order {
    pub pair: String,
    pub side: Side,       // vs. base currency
    pub qty_base: f64,    // size in base units (e.g. EUR)
}

#[derive(Clone, Debug)]
pub struct Fill {
    pub pair: String,
    pub side: Side,
    pub qty_base: f64,
    pub price: f64,       // executed px
}
