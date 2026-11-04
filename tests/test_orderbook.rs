use vortex_engine::generator::SyntheticMarketGenerator;
use vortex_engine::microstructure::MicrostructureAnalyzer;
use vortex_engine::orderbook::OrderBook;
use vortex_engine::types::{OrderBookUpdate, Price, Quantity, Side};
use vortex_engine::VortexEngine;

#[test]
fn test_bids_and_asks_sorting_invariants() {
    let mut book = OrderBook::new("TEST");

    // Add bids: 100.0, 102.5, 101.0
    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 1,
        side: Side::Bid,
        price: Price::from_f64(100.0),
        quantity: Quantity::from_f64(5.0),
    }).unwrap();

    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 2,
        side: Side::Bid,
        price: Price::from_f64(102.5),
        quantity: Quantity::from_f64(2.0),
    }).unwrap();

    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 3,
        side: Side::Bid,
        price: Price::from_f64(101.0),
        quantity: Quantity::from_f64(1.0),
    }).unwrap();

    // Add asks: 105.0, 103.0, 104.0
    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 4,
        side: Side::Ask,
        price: Price::from_f64(105.0),
        quantity: Quantity::from_f64(10.0),
    }).unwrap();

    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 5,
        side: Side::Ask,
        price: Price::from_f64(103.0),
        quantity: Quantity::from_f64(4.0),
    }).unwrap();

    let bbo = book.bbo();

    // Best bid should be highest price (102.5)
    assert_eq!(bbo.best_bid.unwrap().0, Price::from_f64(102.5));
    assert_eq!(bbo.best_bid.unwrap().1, Quantity::from_f64(2.0));

    // Best ask should be lowest price (103.0)
    assert_eq!(bbo.best_ask.unwrap().0, Price::from_f64(103.0));
    assert_eq!(bbo.best_ask.unwrap().1, Quantity::from_f64(4.0));

    // Spread = 103.0 - 102.5 = 0.5
    assert_eq!(bbo.spread.unwrap(), Price::from_f64(0.5));
}

#[test]
fn test_zero_quantity_prunes_price_level() {
    let mut book = OrderBook::new("TEST");

    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 1,
        side: Side::Bid,
        price: Price::from_f64(50.0),
        quantity: Quantity::from_f64(10.0),
    }).unwrap();

    assert_eq!(book.bid_levels_count(), 1);

    // Prune level by sending qty = 0
    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 2,
        side: Side::Bid,
        price: Price::from_f64(50.0),
        quantity: Quantity::ZERO,
    }).unwrap();

    assert_eq!(book.bid_levels_count(), 0);
    assert!(book.bbo().best_bid.is_none());
}

#[test]
fn test_order_book_imbalance_and_micro_price() {
    let mut book = OrderBook::new("TEST");

    // Heavy bid side: Bid 100.0 (Qty 90.0), Ask 102.0 (Qty 10.0)
    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 1,
        side: Side::Bid,
        price: Price::from_f64(100.0),
        quantity: Quantity::from_f64(90.0),
    }).unwrap();

    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 2,
        side: Side::Ask,
        price: Price::from_f64(102.0),
        quantity: Quantity::from_f64(10.0),
    }).unwrap();

    let metrics = MicrostructureAnalyzer::compute(&book, 5);

    // OBI = (90 - 10) / (90 + 10) = 80 / 100 = +0.80 (Strong Bid Heavy)
    assert!((metrics.order_book_imbalance - 0.80).abs() < 1e-4);

    // Mid price = (100 + 102) / 2 = 101.0
    assert_eq!(metrics.bbo.mid_price, Some(101.0));

    // Micro price = (90 * 102 + 10 * 100) / 100 = (9180 + 1000) / 100 = 101.80
    // Micro-price should be higher than mid-price because of heavy bid pressure!
    let mp = metrics.micro_price.unwrap();
    assert!((mp - 101.80).abs() < 1e-4);
    assert!(mp > metrics.bbo.mid_price.unwrap());
}

#[test]
fn test_crossed_order_clearance_matching() {
    let mut book = OrderBook::new("TEST");

    // Resting Bid at 105.0
    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 1,
        side: Side::Bid,
        price: Price::from_f64(105.0),
        quantity: Quantity::from_f64(1.0),
    }).unwrap();

    // Aggressive Ask at 100.0 (crosses and consumes the 105.0 bid)
    book.apply_update(&OrderBookUpdate {
        timestamp_ns: 2,
        side: Side::Ask,
        price: Price::from_f64(100.0),
        quantity: Quantity::from_f64(1.0),
    }).unwrap();

    // Crossing bid was matched/pruned
    assert_eq!(book.bid_levels_count(), 0);
    assert_eq!(book.bbo().best_ask.unwrap().0, Price::from_f64(100.0));
    assert!(!book.is_crossed());
}

#[test]
fn test_high_throughput_execution() {
    let mut engine = VortexEngine::new("BTC-USDT", 5, 10000);
    let mut generator = SyntheticMarketGenerator::new(60000.0, 1.0);

    // Process 25,000 updates
    for _ in 0..25_000 {
        let update = generator.next_update();
        engine.process_update(update);
    }

    assert_eq!(engine.book.total_updates, 25_000);
    assert!(engine.metrics.p50_micros() > 0.0);
    assert!(engine.metrics.throughput_ops_sec() > 10_000.0);
}
