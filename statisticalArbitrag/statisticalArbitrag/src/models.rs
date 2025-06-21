#[derive(Clone, Debug)]
pub struct Tick {
    pub symbol: String,
    pub px:     f64,
    pub ts:     std::time::Instant,
}

#[derive(Clone, Debug)]
pub enum Side { Buy, Sell }

#[derive(Clone, Debug)]
pub struct Order {
    pub symbol: String,
    pub px:     f64,
    pub qty:    f64,
    pub side:   Side,
}

#[derive(Clone, Debug)]
pub struct Fill {
    pub symbol: String,
    pub px:     f64,
    pub qty:    f64,
    pub side:   Side,
}
