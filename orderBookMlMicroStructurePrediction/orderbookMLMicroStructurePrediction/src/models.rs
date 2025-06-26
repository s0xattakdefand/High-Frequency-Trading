// MIT © 2025
//! Shared data types.

use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

/// Snapshot of the top-5 limit-order-book depths plus mid-price.
#[derive(Clone, Debug)]
pub struct Book {
    pub bid_vol: [f64; 5],   // size at bid levels 0-4
    pub ask_vol: [f64; 5],   // size at ask levels 0-4
    pub mid:     f64,
}

#[derive(Clone, Debug)]
pub struct Order {
    pub side: Side,
    pub qty:  f64,
}

#[derive(Clone, Debug)]
pub struct Fill {
    pub px:   f64,
    pub qty:  f64,
    pub side: Side,
    pub ts:   Instant,
}
