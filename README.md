# MangoDB_Coderush1.0
HFT market making model using blockchain


# 🚀 HFT Market Making Bot

A high-frequency trading market making system inspired by Jane Street's trading infrastructure, built in Rust and compiled to WebAssembly for ultra-low latency execution.

## 📊 Overview

This project implements a sophisticated market making bot with real-time L2 order book analysis, volatility clustering detection, and ultra-low latency execution capabilities. The system is designed to provide liquidity in financial markets while managing risk and optimizing for profitability.

## 🏗️ Architecture

### Core Components

1. **Market Maker Engine** - Quote generation and inventory management
2. **Risk Manager** - Real-time risk monitoring and position limits
3. **Volatility Model** - GARCH and EWMA volatility forecasting
4. **Order Book Engine** - Level 2 order book processing
5. **Backtest Engine** - Historical performance simulation
6. **Latency Engine** - Performance monitoring and optimization

## 🧮 Algorithms & Models

### 1. Market Making Algorithm

The core market making strategy implements several sophisticated techniques:

#### Inventory Skew Model
```rust
// Adjusts quotes based on current inventory position
inventory_skew = (current_inventory / max_position_size) * skew_factor
bid_price = mid_price - half_spread + inventory_skew
ask_price = mid_price + half_spread + inventory_skew
```

#### Dynamic Spread Calculation
- **Base Spread**: Target spread in basis points (default: 10 bps)
- **Volatility Adjustment**: `spread += volatility * volatility_factor * 10000`
- **Order Book Adjustment**: Widens spreads when book is thin or imbalanced
- **Adverse Selection Protection**: Increases spreads when detecting toxic flow

#### Quote Size Optimization
```rust
// Adjusts quote sizes based on inventory and volatility
inventory_factor = 1.0 - (abs(inventory) / max_deviation).min(0.8)
volatility_factor = (1.0 / (1.0 + volatility * 5.0)).max(0.3)
adjusted_size = base_size * inventory_factor * volatility_factor
```

### 2. Volatility Models

#### EWMA (Exponentially Weighted Moving Average)
```rust
weighted_var = Σ(decay^i * return_i^2) / Σ(decay^i)
volatility = sqrt(weighted_var) * sqrt(252)  // Annualized
```

#### GARCH(1,1) Model
```rust
σ²(t) = ω + α * ε²(t-1) + β * σ²(t-1)
```
Where:
- `ω` = 0.000001 (constant term)
- `α` = 0.1 (ARCH coefficient)  
- `β` = 0.85 (GARCH coefficient)

#### Volatility Regime Detection
- **Low**: < 10% annualized
- **Normal**: 10-20% annualized
- **High**: 20-40% annualized
- **Extreme**: > 40% annualized

#### Volatility Clustering Score
```rust
// Measures autocorrelation in absolute returns
clustering_score = Σ((|r_t| - μ) * (|r_{t-1}| - μ)) / Σ((|r_t| - μ)²)
```

### 3. Risk Management System

#### Position Limits
- **Maximum Position Size**: $1M per symbol
- **Gross Exposure Limit**: $15M total
- **Net Exposure Limit**: $3M total
- **Concentration Limit**: 30% of portfolio per position

#### Value at Risk (VaR) Calculation
```rust
// Historical simulation method
VaR_95% = portfolio_value * percentile(returns, 5%)
VaR_99% = portfolio_value * percentile(returns, 1%)
```

#### Expected Shortfall (Conditional VaR)
```rust
ES_95% = portfolio_value * mean(returns[returns < VaR_95%])
```

#### Risk Score Calculation
```rust
risk_score = min(100, 
    (VaR_95 / VaR_limit * 25) +
    (drawdown / max_drawdown * 20) +
    (leverage / leverage_limit * 20) +
    (concentration_risk * 15) +
    (gross_exposure / exposure_limit * 10)
)
```

### 4. Order Book Analytics

#### Imbalance Calculation
```rust
imbalance = (total_bid_volume - total_ask_volume) / (total_bid_volume + total_ask_volume)
```

#### Depth Ratio
```rust
depth_ratio = total_bid_depth / total_ask_depth
```

#### Book Pressure
```rust
// Weighted by distance from mid price
bid_pressure = Σ(bid_quantity_i / (level_i + 1))
ask_pressure = Σ(ask_quantity_i / (level_i + 1))
book_pressure = (bid_pressure - ask_pressure) / (bid_pressure + ask_pressure)
```

#### Volume Weighted Average Price (VWAP)
```rust
VWAP = Σ(price_i * volume_i) / Σ(volume_i)
```

### 5. Adverse Selection Detection

#### Fill Rate Analysis
```rust
adverse_rate = adverse_fills / total_fills
if adverse_rate > threshold {
    spread_penalty = (adverse_rate - threshold) * 2.0 * penalty_factor
}
```

#### Implementation
- Monitors recent fills over a sliding window
- Detects when fills consistently move against the market maker
- Automatically widens spreads to compensate for toxic flow

### 6. Latency Optimization

#### Performance Metrics
- **Tick-to-Trade Latency**: Time from market data to order submission
- **Order Book Update Latency**: Time to process L2 updates
- **Quote Generation Latency**: Time to calculate new quotes
- **Execution Latency**: Time from order to fill confirmation

#### Optimization Techniques
- **Cache Warmup**: Pre-loads frequently accessed data
- **Predictive Processing**: Anticipates market movements
- **Async Processing**: Non-blocking operations for critical path
- **FPGA Simulation**: Models hardware acceleration benefits

#### Percentile Tracking
```rust
P50, P95, P99, P99.9 latency measurements
Jitter calculation: sqrt(variance(RTT))
```

### 7. Backtesting Engine

#### Performance Metrics
- **Sharpe Ratio**: `(annual_return - risk_free_rate) / annual_volatility`
- **Sortino Ratio**: `(annual_return - risk_free_rate) / downside_deviation`
- **Calmar Ratio**: `annual_return / max_drawdown`
- **Profit Factor**: `gross_profit / gross_loss`
- **Win Rate**: `winning_trades / total_trades`

#### Transaction Cost Model
```rust
transaction_cost = notional * (transaction_cost_bps + slippage_bps) / 10000
slippage_price = base_price * (1 ± slippage_factor)
```

#### Market Impact Simulation
- Models quote acceptance probability based on market conditions
- Simulates realistic fill rates and market impact
- Accounts for adverse selection in historical simulation

## 🚀 Features

### Real-Time Processing
- WebAssembly compilation for near-native performance
- Sub-millisecond quote generation
- Concurrent order book processing
- Lock-free data structures

### Risk Controls
- Real-time position monitoring
- Dynamic risk limits
- Circuit breakers for extreme market conditions
- Automated position unwinding

### Market Microstructure
- Level 2 order book reconstruction
- Tick-by-tick market data processing
- Market impact modeling
- Liquidity provision optimization

### Performance Analytics
- Real-time P&L tracking
- Latency percentile monitoring
- Risk-adjusted return metrics
- Trade execution analysis

## 🛠️ Technology Stack

- **Language**: Rust (compiled to WebAssembly)
- **Frontend**: Vanilla JavaScript with Chart.js
- **Build System**: wasm-pack
- **Development**: Replit (Nix environment)

## 📈 Getting Started

### Prerequisites
- Rust toolchain with wasm32-unknown-unknown target
- wasm-pack
- Modern web browser with WebAssembly support

### Building
```bash
wasm-pack build --target web --out-dir pkg
```

### Running
```bash
python3 -m http.server 5000
```

Access the application at `http://localhost:5000`

## 🎮 Usage

### Control Panel
- **Start Engine**: Begins market making operations
- **Stop Engine**: Halts all trading activity
- **Run Backtest**: Executes historical performance simulation
- **Reset System**: Clears all state and positions

### Parameters
- **Symbol**: Trading instrument (default: NIFTY50)
- **Quote Size**: Base order quantity
- **Max Position**: Maximum inventory limit
- **Risk Limit**: Maximum loss threshold

### Monitoring
- Real-time price charts
- Order book visualization
- Volatility tracking
- Latency metrics
- P&L curves

## 📊 Performance Benchmarks

Typical performance characteristics:
- **Quote Generation**: < 100 microseconds
- **Risk Check**: < 50 microseconds
- **Order Book Update**: < 10 microseconds
- **Volatility Update**: < 20 microseconds

## 🔧 Configuration

### Market Making Parameters
```rust
target_spread_bps: 10.0,        // Target spread in basis points
min_spread_bps: 2.0,            // Minimum spread
max_spread_bps: 50.0,           // Maximum spread
default_quote_size: 100.0,      // Base order size
skew_factor: 0.5,               // Inventory skew intensity
```

### Risk Parameters
```rust
max_position_size: 1000000.0,   // Maximum position ($1M)
max_daily_loss: 50000.0,        // Daily loss limit ($50K)
var_limit: 100000.0,            // VaR limit ($100K)
leverage_limit: 5.0,            // Maximum leverage ratio
```
### Frontend
<img width="1874" height="918" alt="image" src="https://github.com/user-attachments/assets/cc3b8487-c458-43b8-ae9e-c1cc01c21467" />
### chatbot
<img width="1915" height="923" alt="image" src="https://github.com/user-attachments/assets/be56052e-4447-41ae-9016-18eb57d9161a" />
### analytics
<img width="1882" height="917" alt="image" src="https://github.com/user-attachments/assets/63d4c04e-4df0-42a6-8a25-e6a6df28d926" />
### order book analysis and backtest results
<img width="1804" height="867" alt="image" src="https://github.com/user-attachments/assets/57394c74-6bcc-4fc6-b52c-765e02809101" />
### Risk Metricsa and system logs
<img width="1888" height="644" alt="image" src="https://github.com/user-attachments/assets/e29ed716-9e7e-4390-8ab0-435c4e44477a" />



## 📝 License

This project is for educational and research purposes. Not intended for production trading without proper risk management and regulatory compliance.

## ⚠️ Disclaimer

This software is provided for educational purposes only. Trading financial instruments involves substantial risk of loss. The authors are not responsible for any financial losses incurred through the use of this software.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your changes with tests
4. Submit a pull request

## 📚 References

- Harris, L. (2003). Trading and Exchanges: Market Microstructure for Practitioners
- Hasbrouck, J. (2007). Empirical Market Microstructure
- Cartea, Á., Jaimungal, S., & Penalva, J. (2015). Algorithmic and High-Frequency Trading
- Gatev, E., Goetzmann, W. N., & Rouwenhorst, K. G. (2006). Pairs Trading: Performance of a Relative-Value Arbitrage Rule

---

Built with ❤️ using Rust and WebAssembly
