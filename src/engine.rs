use crate::metrics::LatencyTracker;
use crate::microstructure::MicrostructureAnalyzer;
use crate::orderbook::OrderBook;
use crate::types::{MicrostructureMetrics, OrderBookUpdate};
use crossbeam_channel::{Receiver, Sender};
use std::time::Instant;

/// Core Pipeline Engine.
/// Decouples market data ingestion from state mutation using lock-free crossbeam channels.
pub struct VortexEngine {
    pub book: OrderBook,
    pub metrics: LatencyTracker,
    pub latest_microstructure: MicrostructureMetrics,
    update_rx: Receiver<OrderBookUpdate>,
    update_tx: Sender<OrderBookUpdate>,
    depth_levels: usize,
}

impl VortexEngine {
    pub fn new(symbol: impl Into<String>, depth_levels: usize, channel_capacity: usize) -> Self {
        let (update_tx, update_rx) = crossbeam_channel::bounded(channel_capacity);
        let book = OrderBook::new(symbol);
        let metrics = LatencyTracker::new();

        Self {
            book,
            metrics,
            latest_microstructure: MicrostructureMetrics::default(),
            update_rx,
            update_tx,
            depth_levels,
        }
    }

    /// Ingestion handle to submit updates into the lock-free pipeline from network/feed threads.
    pub fn sender(&self) -> Sender<OrderBookUpdate> {
        self.update_tx.clone()
    }

    /// Process a single update directly (hot-path synchronous processing).
    #[inline(always)]
    pub fn process_update(&mut self, update: OrderBookUpdate) {
        let t0 = Instant::now();

        let _ = self.book.apply_update(&update);
        self.latest_microstructure = MicrostructureAnalyzer::compute(&self.book, self.depth_levels);

        let elapsed_ns = t0.elapsed().as_nanos() as u64;
        self.metrics.record_nanos(elapsed_ns);
    }

    /// Drains available updates from the inbound crossbeam channel.
    pub fn process_pending(&mut self) -> usize {
        let mut count = 0;
        while let Ok(update) = self.update_rx.try_recv() {
            self.process_update(update);
            count += 1;
        }
        count
    }
}
