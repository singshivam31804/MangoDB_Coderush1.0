use crate::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MarketMakerEngine {
    config: MarketMakerConfig,
    state: MarketMakerState,
    skew_engine: InventorySkewEngine,
    adverse_selection_detector: AdverseSelectionDetector,
}

#[derive(Debug, Clone)]
struct MarketMakerConfig {
    target_spread_bps: f64,
    min_spread_bps: f64,
    max_spread_bps: f64,
    default_quote_size: f64,
    max_inventory_deviation: f64,
    skew_factor: f64,
    volatility_adjustment_factor: f64,
    tick_size: f64,
}

#[derive(Debug, Clone)]
struct MarketMakerState {
    current_inventory: HashMap<String, f64>,
    quote_history: Vec<Quote>,
    last_update_time: f64,
    current_volatility: f64,
    pnl_tracker: PnlTracker,
}

#[derive(Debug, Clone)]
struct InventorySkewEngine {
    max_position_size: f64,
    skew_intensity: f64,
    inventory_half_life: f64,
}

#[derive(Debug, Clone)]
struct AdverseSelectionDetector {
    fill_rate_threshold: f64,
    adverse_fill_penalty: f64,
    detection_window: usize,
    recent_fills: Vec<FillEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlTracker {
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub total_volume: f64,
    pub trade_count: u32,
}

#[derive(Debug, Clone)]
struct FillEvent {
    timestamp: f64,
    side: OrderSide,
    price: f64,
    quantity: f64,
    was_adverse: bool,
}

impl MarketMakerEngine {
    pub fn new() -> Self {
        Self {
            config: MarketMakerConfig {
                target_spread_bps: 10.0,
                min_spread_bps: 2.0,
                max_spread_bps: 50.0,
                default_quote_size: 100.0,
                max_inventory_deviation: 1000.0,
                skew_factor: 0.5,
                volatility_adjustment_factor: 2.0,
                tick_size: 0.01,
            },
            state: MarketMakerState {
                current_inventory: HashMap::new(),
                quote_history: Vec::new(),
                last_update_time: 0.0,
                current_volatility: 0.0,
                pnl_tracker: PnlTracker {
                    realized_pnl: 0.0,
                    unrealized_pnl: 0.0,
                    total_volume: 0.0,
                    trade_count: 0,
                },
            },
            skew_engine: InventorySkewEngine {
                max_position_size: 1000.0,
                skew_intensity: 0.3,
                inventory_half_life: 300.0,
            },
            adverse_selection_detector: AdverseSelectionDetector {
                fill_rate_threshold: 0.8,
                adverse_fill_penalty: 2.0,
                detection_window: 50,
                recent_fills: Vec::new(),
            },
        }
    }

    pub fn generate_quotes(
        &mut self,
        market_data: &MarketData,
        order_book: &OrderBook,
        volatility: f64,
    ) -> Vec<Quote> {
        self.state.current_volatility = volatility;
        self.state.last_update_time = market_data.timestamp;

        let symbol = &market_data.symbol;

        // Calculate base spread
        let base_spread = self.calculate_base_spread(volatility, order_book);

        // Apply inventory skew
        let inventory_skew = self.calculate_inventory_skew(symbol);

        // Calculate quote sizes
        let (bid_size, ask_size) = self.calculate_quote_sizes(symbol, volatility);

        // Detect adverse selection and adjust spreads
        let adverse_selection_adjustment = self.detect_adverse_selection();
        let final_spread = base_spread + adverse_selection_adjustment;

        // Calculate mid price
        let mid_price = (market_data.bid_price + market_data.ask_price) / 2.0;

        // Calculate skewed bid/ask prices
        let half_spread = (final_spread / 2.0) * mid_price / 10000.0; // Convert bps to price
        let bid_price = self.round_to_tick(mid_price - half_spread + inventory_skew);
        let ask_price = self.round_to_tick(mid_price + half_spread + inventory_skew);

        // Ensure minimum spread
        let min_spread_price = self.config.min_spread_bps * mid_price / 10000.0;
        let adjusted_ask_price = if ask_price - bid_price < min_spread_price {
            bid_price + min_spread_price
        } else {
            ask_price
        };

        let quote = Quote {
            symbol: symbol.clone(),
            bid_price,
            ask_price: adjusted_ask_price,
            bid_quantity: bid_size,
            ask_quantity: ask_size,
            timestamp: market_data.timestamp,
            confidence: self.calculate_quote_confidence(order_book, volatility),
        };

        // Store quote history
        self.state.quote_history.push(quote.clone());
        if self.state.quote_history.len() > 1000 {
            self.state.quote_history.remove(0);
        }

        vec![quote]
    }

    fn calculate_base_spread(&self, volatility: f64, order_book: &OrderBook) -> f64 {
        // Start with target spread
        let mut spread = self.config.target_spread_bps;

        // Adjust for volatility
        spread += volatility * self.config.volatility_adjustment_factor * 10000.0;

        // Adjust for order book conditions
        spread += self.calculate_order_book_adjustment(order_book);

        // Clamp to min/max bounds
        spread.max(self.config.min_spread_bps).min(self.config.max_spread_bps)
    }

    fn calculate_order_book_adjustment(&self, order_book: &OrderBook) -> f64 {
        let stats = order_book.get_default_stats();

        // Wider spreads when book is thin or imbalanced
        let depth_adjustment = if stats.depth_ratio < 0.5 { 10.0 } else { 0.0 };
        let imbalance_adjustment = stats.imbalance.abs() * 15.0; // Up to 15 bps for severe imbalance

        depth_adjustment + imbalance_adjustment
    }

    fn calculate_inventory_skew(&self, symbol: &str) -> f64 {
        let current_inventory = self.state.current_inventory.get(symbol).copied().unwrap_or(0.0);

        // Calculate inventory ratio relative to max position
        let inventory_ratio = current_inventory / self.skew_engine.max_position_size;

        // Apply skew based on inventory
        let skew_bps = inventory_ratio * self.config.skew_factor * 100.0; // Convert to bps

        // Convert to price adjustment (positive skew = higher quotes to reduce inventory)
        skew_bps / 10000.0 // Very simplified - in reality would use mid price
    }

    fn calculate_quote_sizes(&self, symbol: &str, volatility: f64) -> (f64, f64) {
        let base_size = self.config.default_quote_size;
        let current_inventory = self.state.current_inventory.get(symbol).copied().unwrap_or(0.0);

        // Adjust sizes based on inventory position
        let inventory_factor = 1.0 - (current_inventory.abs() / self.config.max_inventory_deviation).min(0.8);

        // Adjust sizes based on volatility (smaller sizes in high volatility)
        let volatility_factor = (1.0 / (1.0 + volatility * 5.0)).max(0.3);

        let adjusted_size = base_size * inventory_factor * volatility_factor;

        // Skew sizes based on inventory (quote smaller on the side we're long)
        let bid_size = if current_inventory > 0.0 {
            adjusted_size * 0.7 // Reduce bid size when long
        } else {
            adjusted_size
        };

        let ask_size = if current_inventory < 0.0 {
            adjusted_size * 0.7 // Reduce ask size when short
        } else {
            adjusted_size
        };

        (bid_size, ask_size)
    }

    fn detect_adverse_selection(&mut self) -> f64 {
        if self.adverse_selection_detector.recent_fills.len() < 10 {
            return 0.0;
        }

        // Calculate adverse fill rate
        let recent_window = self.adverse_selection_detector.detection_window.min(
            self.adverse_selection_detector.recent_fills.len()
        );

        let recent_fills = &self.adverse_selection_detector.recent_fills[
            self.adverse_selection_detector.recent_fills.len() - recent_window..
        ];

        let adverse_fills = recent_fills.iter().filter(|f| f.was_adverse).count();
        let adverse_rate = adverse_fills as f64 / recent_fills.len() as f64;

        // Apply penalty if adverse selection rate is high
        if adverse_rate > self.adverse_selection_detector.fill_rate_threshold {
            let penalty_factor = (adverse_rate - self.adverse_selection_detector.fill_rate_threshold) * 2.0;
            penalty_factor * self.adverse_selection_detector.adverse_fill_penalty
        } else {
            0.0
        }
    }

    fn calculate_quote_confidence(&self, order_book: &OrderBook, volatility: f64) -> f64 {
        let stats = order_book.get_default_stats();

        // Base confidence
        let mut confidence = 0.8;

        // Reduce confidence in volatile markets
        confidence -= (volatility * 2.0).min(0.3);

        // Reduce confidence when order book is thin
        confidence -= (1.0 - stats.depth_ratio) * 0.2;

        // Reduce confidence when order book is imbalanced
        confidence -= stats.imbalance.abs() * 0.1;

        confidence.max(0.1).min(1.0)
    }

    fn round_to_tick(&self, price: f64) -> f64 {
        (price / self.config.tick_size).round() * self.config.tick_size
    }

    pub fn update_inventory(&mut self, symbol: &str, quantity_change: f64) {
        let current = self.state.current_inventory.get(symbol).copied().unwrap_or(0.0);
        self.state.current_inventory.insert(symbol.to_string(), current + quantity_change);

        console_log!("Inventory updated for {}: {} -> {}", 
                    symbol, current, current + quantity_change);
    }

    pub fn record_fill(&mut self, order: &Order, market_price: f64) {
        // Determine if this was an adverse fill
        let was_adverse = match order.side {
            OrderSide::Buy => order.price > market_price,
            OrderSide::Sell => order.price < market_price,
        };

        let fill_event = FillEvent {
            timestamp: order.timestamp,
            side: order.side.clone(),
            price: order.price,
            quantity: order.quantity,
            was_adverse,
        };

        self.adverse_selection_detector.recent_fills.push(fill_event);

        // Maintain window size
        let max_fills = self.adverse_selection_detector.detection_window * 2;
        if self.adverse_selection_detector.recent_fills.len() > max_fills {
            self.adverse_selection_detector.recent_fills.remove(0);
        }

        // Update PnL tracking
        self.state.pnl_tracker.trade_count += 1;
        self.state.pnl_tracker.total_volume += order.quantity * order.price;
    }

    pub fn get_inventory_summary(&self) -> HashMap<String, f64> {
        self.state.current_inventory.clone()
    }

    pub fn get_pnl_summary(&self) -> PnlTracker {
        self.state.pnl_tracker.clone()
    }

    pub fn reset_inventory(&mut self, symbol: &str) {
        self.state.current_inventory.insert(symbol.to_string(), 0.0);
    }
}