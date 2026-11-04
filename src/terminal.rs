use crate::engine::VortexEngine;
use crossterm::{
    cursor,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::{stdout, Write};

pub struct TerminalDashboard;

impl TerminalDashboard {
    pub fn clear_screen() {
        let mut stdout = stdout();
        let _ = stdout.execute(terminal::Clear(ClearType::All));
        let _ = stdout.execute(cursor::MoveTo(0, 0));
    }

    pub fn render(engine: &VortexEngine, depth_display: usize) {
        let mut stdout = stdout();
        let _ = stdout.execute(cursor::MoveTo(0, 0));
        let clear = "\x1b[K";

        let m = &engine.latest_microstructure;
        let bbo = &m.bbo;

        // 1. Header Banner
        let _ = stdout.execute(SetForegroundColor(Color::Cyan));
        println!("================================================================================{}", clear);
        print!("  ");
        let _ = stdout.execute(SetForegroundColor(Color::White));
        println!("VORTEX-ENGINE : SUB-MICROSECOND L2 ORDER BOOK & MICROSTRUCTURE FEED{}", clear);
        let _ = stdout.execute(SetForegroundColor(Color::DarkGrey));
        println!("  Architecture: Zero-GC Single-Writer (Rust) │ Target: Sub-Microsecond p99{}", clear);
        let _ = stdout.execute(SetForegroundColor(Color::Cyan));
        println!("================================================================================{}", clear);
        let _ = stdout.execute(ResetColor);
        println!("{}", clear);

        // 2. Microstructure & Top-of-Book HUD
        let (bid_str, ask_str) = match (bbo.best_bid, bbo.best_ask) {
            (Some((bp, _)), Some((ap, _))) => (format!("${:.2}", bp.to_f64()), format!("${:.2}", ap.to_f64())),
            _ => ("N/A".to_string(), "N/A".to_string()),
        };

        let spread_str = bbo
            .spread
            .map(|s| format!("${:.4}", s.to_f64()))
            .unwrap_or_else(|| "N/A".to_string());

        let bps_str = bbo
            .spread_bps
            .map(|b| format!("{:.2} bps", b))
            .unwrap_or_else(|| "N/A".to_string());

        let micro_str = m
            .micro_price
            .map(|mp| format!("${:.2}", mp))
            .unwrap_or_else(|| "N/A".to_string());

        print!("  Symbol: ");
        let _ = stdout.execute(SetForegroundColor(Color::Yellow));
        print!("{:<8}", engine.book.symbol);
        let _ = stdout.execute(ResetColor);

        print!(" │ BBO: ");
        let _ = stdout.execute(SetForegroundColor(Color::Green));
        print!("{}", bid_str);
        let _ = stdout.execute(ResetColor);
        print!(" / ");
        let _ = stdout.execute(SetForegroundColor(Color::Red));
        print!("{}", ask_str);
        let _ = stdout.execute(ResetColor);

        print!(" │ Spread: ");
        let _ = stdout.execute(SetForegroundColor(Color::Yellow));
        print!("{} ({})", spread_str, bps_str);
        let _ = stdout.execute(ResetColor);

        print!(" │ Micro-Price: ");
        let _ = stdout.execute(SetForegroundColor(Color::Cyan));
        println!("{}{}", micro_str, clear);
        let _ = stdout.execute(ResetColor);

        // 3. Dynamic Order Book Imbalance (OBI) Gauge
        let obi = m.order_book_imbalance;
        let (gauge_str, label_color, label_text) = Self::render_obi_gauge(obi);

        print!("  OBI: {:+0.3}  {}  ", obi, gauge_str);
        let _ = stdout.execute(SetForegroundColor(label_color));
        println!("{}{}", label_text, clear);
        let _ = stdout.execute(ResetColor);

        // 4. Latency Percentiles Bar
        print!("  Throughput: ");
        let _ = stdout.execute(SetForegroundColor(Color::Yellow));
        print!("{:>7.0} ops/s", engine.metrics.throughput_ops_sec());
        let _ = stdout.execute(ResetColor);

        print!(" │ p50: ");
        let _ = stdout.execute(SetForegroundColor(Color::Green));
        print!("{:>4.1}us", engine.metrics.p50_micros());
        let _ = stdout.execute(ResetColor);

        print!(" │ p90: ");
        let _ = stdout.execute(SetForegroundColor(Color::Cyan));
        print!("{:>4.1}us", engine.metrics.p90_micros());
        let _ = stdout.execute(ResetColor);

        print!(" │ p99: ");
        let _ = stdout.execute(SetForegroundColor(Color::Green));
        print!("{:>4.1}us", engine.metrics.p99_micros());
        let _ = stdout.execute(ResetColor);

        print!(" │ p99.9: ");
        let _ = stdout.execute(SetForegroundColor(Color::Magenta));
        println!("{:>4.1}us{}", engine.metrics.p999_micros(), clear);
        let _ = stdout.execute(ResetColor);

        let _ = stdout.execute(SetForegroundColor(Color::DarkGrey));
        println!("--------------------------------------------------------------------------------{}", clear);
        let _ = stdout.execute(ResetColor);

        // 5. L2 Depth Ladder (Strict column-aligned layout)
        let (top_bids, top_asks) = engine.book.top_depth(depth_display);

        println!("         BIDS (BUY DEPTH)              │              ASKS (SELL DEPTH)        {}", clear);
        println!("  Price (USD)     Qty       Depth      │      Depth       Price (USD)     Qty  {}", clear);
        let _ = stdout.execute(SetForegroundColor(Color::DarkGrey));
        println!("---------------------------------------┼---------------------------------------{}", clear);
        let _ = stdout.execute(ResetColor);

        let max_levels = top_bids.len().max(top_asks.len()).min(depth_display);
        for i in 0..max_levels {
            // Bid side (Green): exactly 39 columns before divider
            if let Some((bp, bq)) = top_bids.get(i) {
                let bar = Self::render_bid_bar(bq.to_f64(), 100.0, 10);
                let _ = stdout.execute(SetForegroundColor(Color::Green));
                print!("  {:>10.4} {:>7.2}   {}      ", bp.to_f64(), bq.to_f64(), bar);
                let _ = stdout.execute(ResetColor);
            } else {
                print!("{:>39}", "");
            }

            print!("│");

            // Ask side (Red): exactly 39 columns after divider
            if let Some((ap, aq)) = top_asks.get(i) {
                let bar = Self::render_ask_bar(aq.to_f64(), 100.0, 10);
                let _ = stdout.execute(SetForegroundColor(Color::Red));
                print!("      {}   {:>10.4} {:>7.2}  ", bar, ap.to_f64(), aq.to_f64());
                let _ = stdout.execute(ResetColor);
            } else {
                print!("{:<39}", "");
            }

            println!("{}", clear);
        }

        let _ = stdout.execute(SetForegroundColor(Color::DarkGrey));
        println!("--------------------------------------------------------------------------------{}", clear);
        let _ = stdout.execute(ResetColor);

        // 6. Footer Telemetry
        print!("  Updates: ");
        let _ = stdout.execute(SetForegroundColor(Color::Yellow));
        print!("{:<10}", engine.book.total_updates);
        let _ = stdout.execute(ResetColor);

        print!(" │ Bids: ");
        let _ = stdout.execute(SetForegroundColor(Color::Green));
        print!("{:<7}", engine.book.bid_levels_count());
        let _ = stdout.execute(ResetColor);

        print!(" │ Asks: ");
        let _ = stdout.execute(SetForegroundColor(Color::Red));
        print!("{:<7}", engine.book.ask_levels_count());
        let _ = stdout.execute(ResetColor);

        print!(" │ State: ");
        let _ = stdout.execute(SetForegroundColor(Color::Green));
        println!("ZERO-GC LOCK-FREE{}", clear);
        let _ = stdout.execute(ResetColor);

        let _ = stdout.execute(SetForegroundColor(Color::Cyan));
        println!("================================================================================{}", clear);
        let _ = stdout.execute(ResetColor);

        let _ = stdout.flush();
    }

    fn render_bid_bar(val: f64, max_val: f64, max_chars: usize) -> String {
        let ratio = (val / max_val).clamp(0.0, 1.0);
        let filled = (ratio * max_chars as f64).round() as usize;
        let mut s = String::with_capacity(max_chars);
        for _ in 0..filled {
            s.push('#');
        }
        for _ in filled..max_chars {
            s.push('-');
        }
        s
    }

    fn render_ask_bar(val: f64, max_val: f64, max_chars: usize) -> String {
        let ratio = (val / max_val).clamp(0.0, 1.0);
        let filled = (ratio * max_chars as f64).round() as usize;
        let mut s = String::with_capacity(max_chars);
        for _ in 0..(max_chars.saturating_sub(filled)) {
            s.push('-');
        }
        for _ in 0..filled {
            s.push('#');
        }
        s
    }

    fn render_obi_gauge(obi: f64) -> (String, Color, String) {
        if obi < -0.05 {
            let ratio = (-obi).clamp(0.0, 1.0);
            let num = (ratio * 10.0).round() as usize;
            let filled = "#".repeat(num);
            let empty = "-".repeat(10 - num);
            let gauge = format!("[ {}{} | ---------- ]", empty, filled);
            let pct = (ratio * 100.0).round() as usize;
            (gauge, Color::Red, format!("({}% Ask Pressure)", pct))
        } else if obi > 0.05 {
            let ratio = obi.clamp(0.0, 1.0);
            let num = (ratio * 10.0).round() as usize;
            let filled = "#".repeat(num);
            let empty = "-".repeat(10 - num);
            let gauge = format!("[ ---------- | {}{} ]", filled, empty);
            let pct = (ratio * 100.0).round() as usize;
            (gauge, Color::Green, format!("({}% Bid Pressure)", pct))
        } else {
            (
                "[ ---------- | ---------- ]".to_string(),
                Color::Yellow,
                "(Equilibrium)".to_string(),
            )
        }
    }
}
