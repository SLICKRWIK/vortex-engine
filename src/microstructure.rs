use crate::orderbook::OrderBook;
use crate::types::{MicrostructureMetrics, Quantity, BBO};

pub struct MicrostructureAnalyzer;

impl MicrostructureAnalyzer {
    /// Computes full quantitative microstructure metrics from an active OrderBook.
    pub fn compute(book: &OrderBook, depth_levels: usize) -> MicrostructureMetrics {
        let bbo: BBO = book.bbo();

        // Extract top-N depth for volume aggregation
        let (top_bids, top_asks) = book.top_depth(depth_levels);

        let sum_bid_qty: u64 = top_bids.iter().map(|(_, q)| q.0).sum();
        let sum_ask_qty: u64 = top_asks.iter().map(|(_, q)| q.0).sum();

        let total_bid_depth = Quantity(sum_bid_qty);
        let total_ask_depth = Quantity(sum_ask_qty);

        // 1. Order Book Imbalance (OBI) on top-N levels
        // OBI = (V_bid - V_ask) / (V_bid + V_ask)
        let total_vol = (sum_bid_qty + sum_ask_qty) as f64;
        let order_book_imbalance = if total_vol > 0.0 {
            (sum_bid_qty as f64 - sum_ask_qty as f64) / total_vol
        } else {
            0.0
        };

        // 2. Volume-Weighted Micro-Price at top-1 level
        // MicroPrice = (V_bid * P_ask + V_ask * P_bid) / (V_bid + V_ask)
        let micro_price = match (bbo.best_bid, bbo.best_ask) {
            (Some((bid_p, bid_q)), Some((ask_p, ask_q))) => {
                let v_bid = bid_q.to_f64();
                let v_ask = ask_q.to_f64();
                let p_bid = bid_p.to_f64();
                let p_ask = ask_p.to_f64();

                let denom = v_bid + v_ask;
                if denom > 0.0 {
                    Some(((v_bid * p_ask) + (v_ask * p_bid)) / denom)
                } else {
                    bbo.mid_price
                }
            }
            _ => None,
        };

        MicrostructureMetrics {
            bbo,
            micro_price,
            order_book_imbalance,
            total_bid_depth,
            total_ask_depth,
        }
    }
}
