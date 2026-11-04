use serde::{Deserialize, Serialize};
use std::fmt;

pub const PRICE_SCALE: f64 = 100_000.0;
pub const QTY_SCALE: f64 = 10_000.0;

/// Fixed-point Price representation (5 decimal places: 1 unit = $0.00001).
/// Eliminates floating-point non-determinism on the hot path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct Price(pub u64);

impl Price {
    pub const ZERO: Price = Price(0);

    #[inline(always)]
    pub fn from_f64(val: f64) -> Self {
        if val <= 0.0 {
            Price(0)
        } else {
            Price((val * PRICE_SCALE).round() as u64)
        }
    }

    #[inline(always)]
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / PRICE_SCALE
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.to_f64())
    }
}

/// Fixed-point Quantity representation (4 decimal places: 1 unit = 0.0001 shares/contracts/tokens).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct Quantity(pub u64);

impl Quantity {
    pub const ZERO: Quantity = Quantity(0);

    #[inline(always)]
    pub fn from_f64(val: f64) -> Self {
        if val <= 0.0 {
            Quantity(0)
        } else {
            Quantity((val * QTY_SCALE).round() as u64)
        }
    }

    #[inline(always)]
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / QTY_SCALE
    }

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.to_f64())
    }
}

/// Trade Side (Bid = Buy, Ask = Sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Bid => write!(f, "BID"),
            Side::Ask => write!(f, "ASK"),
        }
    }
}

/// Streaming Level-2 depth update.
/// If quantity is zero, the price level is pruned from the order book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderBookUpdate {
    pub timestamp_ns: u64,
    pub side: Side,
    pub price: Price,
    pub quantity: Quantity,
}

/// Top of the book snapshot (Best Bid & Offer).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct BBO {
    pub best_bid: Option<(Price, Quantity)>,
    pub best_ask: Option<(Price, Quantity)>,
    pub spread: Option<Price>,
    pub spread_bps: Option<f64>,
    pub mid_price: Option<f64>,
}

/// Quantitative microstructure metrics computed deterministically on each book update.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MicrostructureMetrics {
    pub bbo: BBO,
    pub micro_price: Option<f64>,
    pub order_book_imbalance: f64, // Range: -1.0 (100% Ask heavy) to +1.0 (100% Bid heavy)
    pub total_bid_depth: Quantity,
    pub total_ask_depth: Quantity,
}
