use crate::types::{OrderBookUpdate, Price, Quantity, Side};
use std::time::{SystemTime, UNIX_EPOCH};

/// Fast pseudo-random number generator (Xorshift64star) for zero-dependency high-speed tick generation.
struct FastRng {
    state: u64,
}

impl FastRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545f4914f6cdd1d)
    }

    #[inline(always)]
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    #[inline(always)]
    fn range_f64(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }
}

/// Generates realistic synthetic L2 market depth deltas with mean-reverting price walk.
pub struct SyntheticMarketGenerator {
    initial_mid: f64,
    current_mid: f64,
    spread: f64,
    rng: FastRng,
}

impl SyntheticMarketGenerator {
    pub fn new(initial_mid: f64, spread: f64) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(123456789);

        Self {
            initial_mid,
            current_mid: initial_mid,
            spread,
            rng: FastRng::new(seed),
        }
    }

    /// Generates a single market delta update.
    #[inline(always)]
    pub fn next_update(&mut self) -> OrderBookUpdate {
        // Mean-reverting Ornstein-Uhlenbeck walk around initial mid
        let reversion = 0.01 * (self.initial_mid - self.current_mid);
        let noise = (self.rng.next_f64() - 0.5) * (self.spread * 0.4);
        self.current_mid = (self.current_mid + reversion + noise).max(1.0);

        let is_bid = self.rng.next_u64() % 2 == 0;
        let side = if is_bid { Side::Bid } else { Side::Ask };

        // Distance from mid: 0 to 20 tick levels
        let tick_offset = (self.rng.next_u64() % 20) as f64 * (self.spread * 0.5);
        let half_spread = self.spread / 2.0;

        let raw_price = if is_bid {
            self.current_mid - half_spread - tick_offset
        } else {
            self.current_mid + half_spread + tick_offset
        };

        // 10% probability of level cancellation (qty = 0)
        let is_cancel = (self.rng.next_u64() % 100) < 10;
        let qty = if is_cancel {
            0.0
        } else {
            self.rng.range_f64(0.5, 50.0)
        };

        let now_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        OrderBookUpdate {
            timestamp_ns: now_ns,
            side,
            price: Price::from_f64(raw_price),
            quantity: Quantity::from_f64(qty),
        }
    }
}
