
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

// Import all modules
mod order_book;
mod market_maker;
mod risk_manager;
mod volatility;
mod backtest;
mod latency_engine;

// Re-export all public items
pub use order_book::*;
pub use market_maker::*;
pub use risk_manager::*;
pub use volatility::*;
pub use backtest::*;
pub use latency_engine::*;

// Console logging macro
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format!($($t)*).into()))
}

// Utility function to get current timestamp
pub fn now() -> f64 {
    js_sys::Date::now()
}

// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

// Core data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub timestamp: f64,
    pub last_price: f64,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub symbol: String,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_quantity: f64,
    pub ask_quantity: f64,
    pub timestamp: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub timestamp: f64,
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub average_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub price: f64,
    pub quantity: f64,
    pub timestamp: f64,
}

// Main trading engine that combines all components
#[wasm_bindgen]
pub struct HFTEngine {
    order_book: OrderBook,
    market_maker: MarketMakerEngine,
    risk_manager: RiskManager,
    volatility_model: VolatilityModel,
    backtest_engine: BacktestEngine,
    latency_engine: LatencyEngine,
    positions: HashMap<String, Position>,
    current_time: f64,
}

#[wasm_bindgen]
impl HFTEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> HFTEngine {
        console_log!("Initializing HFT Engine");
        
        HFTEngine {
            order_book: OrderBook::new(),
            market_maker: MarketMakerEngine::new(),
            risk_manager: RiskManager::new(),
            volatility_model: VolatilityModel::new(),
            backtest_engine: BacktestEngine::new(),
            latency_engine: LatencyEngine::new(),
            positions: HashMap::new(),
            current_time: 0.0,
        }
    }

    #[wasm_bindgen]
    pub fn process_market_data(&mut self, data: JsValue) -> JsValue {
        let start_time = now();
        
        let market_data: MarketData = serde_wasm_bindgen::from_value(data).unwrap();
        self.current_time = market_data.timestamp;
        
        // Update order book
        self.order_book.update(&market_data);
        
        // Update volatility model
        let volatility = self.volatility_model.update(market_data.last_price, market_data.timestamp);
        
        // Generate quotes
        let quotes = self.market_maker.generate_quotes(
            &market_data,
            &self.order_book,
            volatility,
        );
        
        // Evaluate risk
        let risk_metrics = self.risk_manager.evaluate_risk(&self.positions, &quotes);
        
        // Record latency
        let processing_time = now() - start_time;
        self.latency_engine.record_latency(processing_time);
        
        let response = ProcessingResult {
            quotes: quotes.clone(),
            risk_metrics,
            volatility,
            order_book_stats: self.order_book.get_default_stats(),
            latency_stats: self.latency_engine.get_stats(),
        };
        
        serde_wasm_bindgen::to_value(&response).unwrap()
    }

    #[wasm_bindgen]
    pub fn run_backtest(&mut self, historical_data: JsValue) -> JsValue {
        console_log!("Starting backtest");
        
        let data: Vec<MarketData> = serde_wasm_bindgen::from_value(historical_data).unwrap();
        
        let results = self.backtest_engine.run_backtest(
            data,
            &mut self.market_maker,
            &mut self.risk_manager,
            &mut self.volatility_model,
        );
        
        serde_wasm_bindgen::to_value(&results).unwrap()
    }

    #[wasm_bindgen]
    pub fn get_performance_metrics(&self) -> JsValue {
        let metrics = PerformanceMetrics {
            total_trades: self.positions.len() as u32,
            current_positions: self.positions.len() as u32,
            total_pnl: self.positions.values().map(|p| p.realized_pnl + p.unrealized_pnl).sum(),
            latency_stats: self.latency_engine.get_stats(),
            risk_metrics: RiskMetrics {
                var_95: 0.0,
                var_99: 0.0,
                expected_shortfall: 0.0,
                max_drawdown: 0.0,
                gross_exposure: 0.0,
                net_exposure: 0.0,
                leverage: 0.0,
                concentration_risk: 0.0,
                risk_score: 0.0,
            },
        };
        
        serde_wasm_bindgen::to_value(&metrics).unwrap()
    }

    #[wasm_bindgen]
    pub fn benchmark_performance(&mut self) -> JsValue {
        let benchmark = self.latency_engine.benchmark_processing_pipeline();
        serde_wasm_bindgen::to_value(&benchmark).unwrap()
    }

    #[wasm_bindgen]
    pub fn simulate_fpga_acceleration(&mut self) -> f64 {
        self.latency_engine.simulate_fpga_acceleration()
    }

    #[wasm_bindgen]
    pub fn simulate_gpu_acceleration(&mut self) -> f64 {
        self.latency_engine.simulate_gpu_acceleration()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub quotes: Vec<Quote>,
    pub risk_metrics: RiskMetrics,
    pub volatility: f64,
    pub order_book_stats: OrderBookStats,
    pub latency_stats: LatencyStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_trades: u32,
    pub current_positions: u32,
    pub total_pnl: f64,
    pub latency_stats: LatencyStats,
    pub risk_metrics: RiskMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookStats {
    pub bid_ask_spread: f64,
    pub mid_price: f64,
    pub imbalance: f64,
    pub depth_ratio: f64,
    pub book_pressure: f64,
}


