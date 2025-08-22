# 🚀 HFT Market Making Bot - Setup Guide

## Prerequisites

1. **Node.js** (v16 or higher)
   - Download from: https://nodejs.org/
   - Verify installation: `node --version`

2. **Rust** (with wasm32 target)
   - Install Rust: https://rustup.rs/
   - Add WebAssembly target: `rustup target add wasm32-unknown-unknown`

3. **wasm-pack**
   - Install: `cargo install wasm-pack`

## Quick Start

### Option 1: Using the batch file (Windows)
```bash
# Simply double-click or run:
start.bat
```

### Option 2: Manual setup
```bash
# 1. Install dependencies
npm install

# 2. Build WebAssembly module
wasm-pack build --target web --out-dir pkg

# 3. Start the server
npm start
```

## Accessing the Application

1. Open your web browser
2. Navigate to: `http://localhost:5000`
3. The HFT trading interface will load automatically

## Features Available

### 🎮 Engine Controls
- **Start Engine**: Begin real-time market making operations
- **Stop Engine**: Halt all trading activity
- **Run Backtest**: Execute historical performance simulation
- **Reset System**: Clear all state and positions

### ⚙️ Parameters
- **Symbol**: Trading instrument (default: NIFTY50)
- **Quote Size**: Base order quantity
- **Max Position**: Maximum inventory limit
- **Risk Limit**: Maximum loss threshold

### 🔧 Latency Optimization
- **Benchmark Performance**: Measure system performance
- **Simulate FPGA**: Model hardware acceleration benefits
- **Simulate GPU**: Model parallel processing benefits

### 📊 Market Data Generator
- **Generate Sample Data**: Create synthetic market data
- **Load Historical Data**: Load real market data (future feature)
- **Data Points**: Number of historical data points for backtesting
- **Volatility**: Market volatility level for simulation

## Troubleshooting

### Common Issues

1. **"Module not found" errors**
   - Run `npm install` to install dependencies
   - Ensure you're in the correct directory

2. **WebAssembly build fails**
   - Verify Rust installation: `rustc --version`
   - Check wasm32 target: `rustup target list --installed`
   - Install wasm-pack: `cargo install wasm-pack`

3. **Server won't start**
   - Check if port 5000 is already in use
   - Try a different port by modifying `server.js`

4. **Charts not loading**
   - Check browser console for JavaScript errors
   - Ensure Chart.js CDN is accessible

### Port Already in Use
If port 5000 is busy, modify `server.js`:
```javascript
const PORT = 5001; // Change to any available port
```

## Development

### File Structure
```
Shaan/
├── src/                    # Rust source code
│   ├── lib.rs             # Main library
│   ├── market_maker.rs    # Market making engine
│   ├── order_book.rs      # Order book management
│   ├── risk_manager.rs    # Risk management
│   ├── volatility.rs      # Volatility models
│   ├── backtest.rs        # Backtesting engine
│   └── latency_engine.rs  # Performance monitoring
├── pkg/                   # WebAssembly output
├── index.html             # Main interface
├── script.js              # Frontend JavaScript
├── server.js              # Node.js server
├── package.json           # Node.js dependencies
└── Cargo.toml            # Rust dependencies
```

### Building for Production
```bash
# Build optimized WebAssembly
wasm-pack build --target web --out-dir pkg --release

# Start production server
NODE_ENV=production npm start
```

## Support

If you encounter issues:
1. Check the browser console for errors
2. Verify all prerequisites are installed
3. Ensure all dependencies are up to date
4. Check the system log in the application interface

---

**Note**: This is a simulation system for educational purposes. Not intended for actual trading without proper risk management and regulatory compliance.
