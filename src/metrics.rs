use hdrhistogram::Histogram;
use std::time::Instant;

/// High Dynamic Range Latency Tracker for sub-microsecond tick analysis.
pub struct LatencyTracker {
    // Histogram recording nanosecond latencies up to 100 seconds with 3 sig figs
    histogram: Histogram<u64>,
    start_time: Instant,
    total_events: u64,
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyTracker {
    pub fn new() -> Self {
        // Range: 1 ns to 60,000,000,000 ns (60s), 3 significant figures
        let histogram = Histogram::<u64>::new_with_bounds(1, 60_000_000_000, 3)
            .expect("Failed to initialize HDR histogram");

        Self {
            histogram,
            start_time: Instant::now(),
            total_events: 0,
        }
    }

    #[inline(always)]
    pub fn record_nanos(&mut self, nanos: u64) {
        let val = nanos.max(1);
        let _ = self.histogram.record(val);
        self.total_events += 1;
    }

    /// Latency percentiles in microseconds (μs).
    pub fn p50_micros(&self) -> f64 {
        self.histogram.value_at_quantile(0.50) as f64 / 1_000.0
    }

    pub fn p90_micros(&self) -> f64 {
        self.histogram.value_at_quantile(0.90) as f64 / 1_000.0
    }

    pub fn p99_micros(&self) -> f64 {
        self.histogram.value_at_quantile(0.99) as f64 / 1_000.0
    }

    pub fn p999_micros(&self) -> f64 {
        self.histogram.value_at_quantile(0.999) as f64 / 1_000.0
    }

    pub fn max_micros(&self) -> f64 {
        self.histogram.max() as f64 / 1_000.0
    }

    pub fn total_samples(&self) -> u64 {
        self.total_events
    }

    pub fn throughput_ops_sec(&self) -> f64 {
        let elapsed_sec = self.start_time.elapsed().as_secs_f64();
        if elapsed_sec > 0.0 {
            self.total_events as f64 / elapsed_sec
        } else {
            0.0
        }
    }

    pub fn reset(&mut self) {
        self.histogram.reset();
        self.start_time = Instant::now();
        self.total_events = 0;
    }
}
