use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vortex_engine::generator::SyntheticMarketGenerator;
use vortex_engine::microstructure::MicrostructureAnalyzer;
use vortex_engine::orderbook::OrderBook;

fn benchmark_orderbook_updates(c: &mut Criterion) {
    let mut generator = SyntheticMarketGenerator::new(65000.0, 1.0);
    let updates: Vec<_> = (0..5000).map(|_| generator.next_update()).collect();

    c.bench_function("orderbook_apply_update_5k", |b| {
        b.iter_batched(
            || (OrderBook::new("BTC-USDT"), updates.clone()),
            |(mut book, batch)| {
                for update in batch {
                    let _ = black_box(book.apply_update(&update));
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    c.bench_function("microstructure_compute_top5", |b| {
        let mut book = OrderBook::new("BTC-USDT");
        for u in &updates[0..200] {
            let _ = book.apply_update(u);
        }
        b.iter(|| {
            black_box(MicrostructureAnalyzer::compute(&book, 5));
        });
    });
}

criterion_group!(benches, benchmark_orderbook_updates);
criterion_main!(benches);
