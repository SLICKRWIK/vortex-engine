use crate::types::{OrderBookUpdate, Price, Quantity, Side, BBO};
use std::cmp::Reverse;
use std::collections::BTreeMap;

/// High-Performance In-Memory Level-2 Order Book.
/// Maintains sorted bid and ask ladders with sub-microsecond insertion/deletion.
#[derive(Debug, Clone)]
pub struct OrderBook {
    pub symbol: String,
    // Bids sorted descending: Reverse(Price) ensures highest price is at iterator front
    bids: BTreeMap<Reverse<Price>, Quantity>,
    // Asks sorted ascending: lowest price is at iterator front
    asks: BTreeMap<Price, Quantity>,
    pub total_updates: u64,
}

impl OrderBook {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            total_updates: 0,
        }
    }

    /// Applies a delta update into the book.
    /// If quantity == 0, the price level is pruned.
    /// Invariant: Clears opposing crossed levels on aggressive price updates,
    /// reconstructing consistent depth state in the event of aggressive fills.
    #[inline(always)]
    pub fn apply_update(&mut self, update: &OrderBookUpdate) -> Result<(), &'static str> {
        match update.side {
            Side::Bid => {
                let key = Reverse(update.price);
                if update.quantity.is_zero() {
                    self.bids.remove(&key);
                } else {
                    // Match / prune crossed asks (simulating trade fills on crossing)
                    while let Some((&ask_p, _)) = self.asks.iter().next() {
                        if update.price >= ask_p {
                            self.asks.remove(&ask_p);
                        } else {
                            break;
                        }
                    }
                    self.bids.insert(key, update.quantity);
                }
            }
            Side::Ask => {
                if update.quantity.is_zero() {
                    self.asks.remove(&update.price);
                } else {
                    // Match / prune crossed bids (simulating trade fills on crossing)
                    while let Some((&Reverse(bid_p), _)) = self.bids.iter().next() {
                        if update.price <= bid_p {
                            self.bids.remove(&Reverse(bid_p));
                        } else {
                            break;
                        }
                    }
                    self.asks.insert(update.price, update.quantity);
                }
            }
        }
        self.total_updates += 1;
        Ok(())
    }

    /// Returns the top-of-book (Best Bid & Offer).
    #[inline(always)]
    pub fn bbo(&self) -> BBO {
        let best_bid = self.bids.iter().next().map(|(r_price, qty)| (r_price.0, *qty));
        let best_ask = self.asks.iter().next().map(|(price, qty)| (*price, *qty));

        let mut spread = None;
        let mut spread_bps = None;
        let mut mid_price = None;

        if let (Some((bid_p, _)), Some((ask_p, _))) = (best_bid, best_ask) {
            if ask_p >= bid_p {
                let s = Price(ask_p.0 - bid_p.0);
                let mid = (bid_p.to_f64() + ask_p.to_f64()) / 2.0;
                let bps = if mid > 0.0 {
                    (s.to_f64() / mid) * 10_000.0
                } else {
                    0.0
                };
                spread = Some(s);
                spread_bps = Some(bps);
                mid_price = Some(mid);
            }
        }

        BBO {
            best_bid,
            best_ask,
            spread,
            spread_bps,
            mid_price,
        }
    }

    /// Checks if the book is crossed (Best Bid >= Best Ask).
    #[inline(always)]
    pub fn is_crossed(&self) -> bool {
        match (self.bids.iter().next(), self.asks.iter().next()) {
            (Some((Reverse(bid_p), _)), Some((ask_p, _))) => bid_p >= ask_p,
            _ => false,
        }
    }

    /// Extracts top N levels for bids and asks.
    pub fn top_depth(&self, levels: usize) -> (Vec<(Price, Quantity)>, Vec<(Price, Quantity)>) {
        let top_bids = self
            .bids
            .iter()
            .take(levels)
            .map(|(r_p, q)| (r_p.0, *q))
            .collect();

        let top_asks = self
            .asks
            .iter()
            .take(levels)
            .map(|(p, q)| (*p, *q))
            .collect();

        (top_bids, top_asks)
    }

    /// Total active bid price levels.
    #[inline(always)]
    pub fn bid_levels_count(&self) -> usize {
        self.bids.len()
    }

    /// Total active ask price levels.
    #[inline(always)]
    pub fn ask_levels_count(&self) -> usize {
        self.asks.len()
    }

    /// Clears the entire book state.
    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.total_updates = 0;
    }
}
