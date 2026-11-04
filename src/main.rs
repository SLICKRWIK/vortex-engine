use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use vortex_engine::generator::SyntheticMarketGenerator;
use vortex_engine::terminal::TerminalDashboard;
use vortex_engine::VortexEngine;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Catch Ctrl+C gracefully
    let _ = ctrlc_setup(r);

    let symbol = "BTC-USDT";
    let depth_display = 8;
    let mut engine = VortexEngine::new(symbol, 10, 65536);
    let mut generator = SyntheticMarketGenerator::new(65000.0, 1.50);

    TerminalDashboard::clear_screen();
    println!("Initializing VortexEngine on {}...", symbol);

    // Warm-up initial book depth (populate 50 levels on each side)
    for _ in 0..100 {
        let update = generator.next_update();
        engine.process_update(update);
    }

    let mut last_render = Instant::now();
    let render_interval = Duration::from_millis(40); // ~25 FPS visual refresh
    let start_all = Instant::now();

    // Run high-frequency simulation loop
    while running.load(Ordering::Relaxed) {
        // Process a burst of 500 updates on the hot-path
        for _ in 0..500 {
            let update = generator.next_update();
            engine.process_update(update);
        }

        // Periodic terminal UI update
        if last_render.elapsed() >= render_interval {
            TerminalDashboard::render(&engine, depth_display);
            last_render = Instant::now();
            thread::sleep(Duration::from_micros(200));
        }

        // Demo wraps up after 8 seconds if not interrupted (plenty for high-FPS visual capture)
        if start_all.elapsed().as_secs() >= 8 {
            break;
        }
    }

    // Final Post-Run Summary
    println!("\n================================================================================");
    println!("  ⚡ VORTEX-ENGINE BENCHMARK SUMMARY REPORT ⚡");
    println!("================================================================================");
    println!(" Symbol                : {}", symbol);
    println!(" Total Updates Tested  : {:>12}", engine.book.total_updates);
    println!(" Sustained Throughput  : {:>12.0} updates/sec", engine.metrics.throughput_ops_sec());
    println!(" Median Latency (p50)  : {:>12.2} μs", engine.metrics.p50_micros());
    println!(" 90th Percentile (p90) : {:>12.2} μs", engine.metrics.p90_micros());
    println!(" 99th Percentile (p99) : {:>12.2} μs", engine.metrics.p99_micros());
    println!(" 99.9th Tail (p99.9)   : {:>12.2} μs", engine.metrics.p999_micros());
    println!(" Maximum Jitter        : {:>12.2} μs", engine.metrics.max_micros());
    println!("================================================================================\n");
}

fn ctrlc_setup(running: Arc<AtomicBool>) -> Result<(), ctrlc::Error> {
    ctrlc::set_handler(move || {
        running.store(false, Ordering::SeqCst);
    })
}
