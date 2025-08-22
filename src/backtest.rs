
use crate::*;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct BacktestEngine {
    config: BacktestConfig,
    results: BacktestResults,
    trade_history: Vec<Trade>,
    pnl_history: VecDeque<f64>,
    drawdown_history: VecDeque<f64>,
}

#[derive(Debug, Clone)]
struct BacktestConfig {
    initial_capital: f64,
    transaction_cost_bps: f64,
    slippage_bps: f64,
    max_lookback_days: usize,
    benchmark_symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResults {
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub total_trades: u32,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub avg_trade_pnl: f64,
    pub volatility: f64,
    pub calmar_ratio: f64,
    pub sortino_ratio: f64,
    pub final_capital: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    symbol: String,
    side: OrderSide,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    entry_time: f64,
    exit_time: f64,
    pnl: f64,
    transaction_costs: f64,
}

impl BacktestEngine {
    pub fn new() -> Self {
        Self {
            config: BacktestConfig {
                initial_capital: 1000000.0, // $1M starting capital
                transaction_cost_bps: 2.0,  // 2 bps transaction cost
                slippage_bps: 1.0,          // 1 bps slippage
                max_lookback_days: 252,     // 1 year of trading days
                benchmark_symbol: "NIFTY50".to_string(),
            },
            results: BacktestResults {
                total_return: 0.0,
                sharpe_ratio: 0.0,
                max_drawdown: 0.0,
                total_trades: 0,
                win_rate: 0.0,
                profit_factor: 0.0,
                avg_trade_pnl: 0.0,
                volatility: 0.0,
                calmar_ratio: 0.0,
                sortino_ratio: 0.0,
                final_capital: 0.0,
            },
            trade_history: Vec::new(),
            pnl_history: VecDeque::new(),
            drawdown_history: VecDeque::new(),
        }
    }

    pub fn run_backtest(
        &mut self,
        historical_data: Vec<MarketData>,
        market_maker: &mut MarketMakerEngine,
        risk_manager: &mut RiskManager,
        volatility_model: &mut VolatilityModel,
    ) -> BacktestResults {
        console_log!("Starting backtest with {} data points", historical_data.len());
        
        let mut current_capital = self.config.initial_capital;
        let mut positions: HashMap<String, Position> = HashMap::new();
        let mut order_book = OrderBook::new();
        let mut daily_pnls = Vec::new();
        let mut peak_capital = current_capital;
        let mut max_drawdown: f64 = 0.0;
        
        for (i, market_data) in historical_data.iter().enumerate() {
            // Update volatility model
            let volatility = volatility_model.update(market_data.last_price, market_data.timestamp);
            
            // Update order book with market data
            order_book.update(market_data);
            
            // Generate quotes from market maker
            let quotes = market_maker.generate_quotes(market_data, &order_book, volatility);
            
            // Simulate market making activity
            if i > 50 { // Allow warm-up period
                self.simulate_market_making_round(
                    market_data,
                    &quotes,
                    &mut positions,
                    &mut current_capital,
                    market_maker,
                    risk_manager,
                );
            }
            
            // Calculate daily PnL and drawdown
            if i > 0 && i % 100 == 0 { // Every 100 ticks simulate a day
                let daily_pnl = self.calculate_portfolio_pnl(&positions, market_data);
                daily_pnls.push(daily_pnl);
                current_capital += daily_pnl;
                
                // Update peak and drawdown
                if current_capital > peak_capital {
                    peak_capital = current_capital;
                } else {
                    let drawdown = (peak_capital - current_capital) / peak_capital;
                    max_drawdown = max_drawdown.max(drawdown);
                }
                
                self.pnl_history.push_back(daily_pnl);
                self.drawdown_history.push_back((peak_capital - current_capital) / peak_capital);
                
                // Maintain history size
                if self.pnl_history.len() > self.config.max_lookback_days {
                    self.pnl_history.pop_front();
                }
                if self.drawdown_history.len() > self.config.max_lookback_days {
                    self.drawdown_history.pop_front();
                }
            }
        }
        
        // Calculate final results
        self.results = self.calculate_performance_metrics(current_capital, &daily_pnls, max_drawdown);
        
        console_log!("Backtest completed: Total Return: {:.2}%, Sharpe: {:.3}, Max DD: {:.2}%", 
                    self.results.total_return * 100.0, 
                    self.results.sharpe_ratio, 
                    self.results.max_drawdown * 100.0);
        
        self.results.clone()
    }

    fn simulate_market_making_round(
        &mut self,
        market_data: &MarketData,
        quotes: &[Quote],
        positions: &mut HashMap<String, Position>,
        current_capital: &mut f64,
        market_maker: &mut MarketMakerEngine,
        risk_manager: &RiskManager,
    ) {
        // Simulate quote acceptance/rejection based on market conditions
        let acceptance_probability = self.calculate_quote_acceptance_probability(market_data);
        
        for quote in quotes {
            if self.should_accept_quote(acceptance_probability) {
                // Simulate a fill
                let (side, price, quantity) = if quote.bid_price > 0.0 {
                    (OrderSide::Sell, quote.bid_price, quote.bid_quantity) // Someone hits our bid
                } else {
                    (OrderSide::Buy, quote.ask_price, quote.ask_quantity) // Someone lifts our offer
                };
                
                // Create simulated order
                let order = Order {
                    id: format!("sim_{}", now()),
                    symbol: quote.symbol.clone(),
                    side: side.clone(),
                    quantity,
                    price,
                    timestamp: market_data.timestamp,
                    order_type: OrderType::Market,
                };
                
                // Check risk limits
                if risk_manager.validate_order(&order, positions) {
                    // Execute the trade
                    self.execute_simulated_trade(&order, positions, current_capital, market_maker);
                }
            }
        }
    }

    fn calculate_quote_acceptance_probability(&self, market_data: &MarketData) -> f64 {
        // Base probability
        let mut probability = 0.1; // 10% base chance
        
        // Higher probability with higher volume
        probability += (market_data.volume / 10000.0).min(0.2);
        
        // Higher probability with wider spreads (more attractive quotes)
        let spread = market_data.ask_price - market_data.bid_price;
        let spread_ratio = spread / market_data.bid_price;
        probability += (spread_ratio * 100.0).min(0.3);
        
        probability.min(0.8) // Cap at 80%
    }

    fn should_accept_quote(&self, probability: f64) -> bool {
        // Simple random number generation simulation
        let random_factor = (now() % 1000.0) / 1000.0;
        random_factor < probability
    }

    fn execute_simulated_trade(
        &mut self,
        order: &Order,
        positions: &mut HashMap<String, Position>,
        _current_capital: &mut f64,
        market_maker: &mut MarketMakerEngine,
    ) {
        // Calculate transaction costs
        let notional = order.quantity * order.price;
        let transaction_cost = notional * (self.config.transaction_cost_bps + self.config.slippage_bps) / 10000.0;
        
        // Apply slippage to price
        let slippage_factor = self.config.slippage_bps / 10000.0;
        let execution_price = match order.side {
            OrderSide::Buy => order.price * (1.0 + slippage_factor),
            OrderSide::Sell => order.price * (1.0 - slippage_factor),
        };
        
        // Update position
        let position = positions.entry(order.symbol.clone()).or_insert(Position {
            symbol: order.symbol.clone(),
            quantity: 0.0,
            average_price: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        });
        
        let quantity_change = match order.side {
            OrderSide::Buy => order.quantity,
            OrderSide::Sell => -order.quantity,
        };
        
        // Calculate realized PnL if closing position
        let mut realized_pnl = 0.0;
        if (position.quantity > 0.0 && quantity_change < 0.0) || 
           (position.quantity < 0.0 && quantity_change > 0.0) {
            let closing_quantity = quantity_change.abs().min(position.quantity.abs());
            realized_pnl = (execution_price - position.average_price) * closing_quantity *
                          if position.quantity > 0.0 { 1.0 } else { -1.0 };
        }
        
        // Update position
        if (position.quantity + quantity_change).abs() < 0.001 {
            // Position closed
            position.realized_pnl += realized_pnl - transaction_cost;
            position.quantity = 0.0;
            position.average_price = 0.0;
        } else {
            // Update average price for remaining position
            let new_quantity = position.quantity + quantity_change;
            if new_quantity.signum() == quantity_change.signum() {
                // Adding to position
                let total_cost = position.quantity * position.average_price + quantity_change * execution_price;
                position.average_price = total_cost / new_quantity;
            }
            position.quantity = new_quantity;
            position.realized_pnl += realized_pnl - transaction_cost;
        }
        
        // Record trade
        let trade = Trade {
            symbol: order.symbol.clone(),
            side: order.side.clone(),
            entry_price: execution_price,
            exit_price: 0.0, // Will be updated when position is closed
            quantity: order.quantity,
            entry_time: order.timestamp,
            exit_time: 0.0,
            pnl: realized_pnl - transaction_cost,
            transaction_costs: transaction_cost,
        };
        
        self.trade_history.push(trade);
        
        // Update market maker inventory
        market_maker.update_inventory(&order.symbol, quantity_change);
        
        console_log!("Trade executed: {} {} {:.0}@{:.2}, PnL: {:.2}", 
                    order.symbol, 
                    match order.side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" },
                    order.quantity, execution_price, realized_pnl - transaction_cost);
    }

    fn calculate_portfolio_pnl(&self, positions: &HashMap<String, Position>, market_data: &MarketData) -> f64 {
        let mut total_pnl = 0.0;
        
        for position in positions.values() {
            total_pnl += position.realized_pnl;
            
            // Calculate unrealized PnL using current market price
            if position.quantity.abs() > 0.001 {
                let unrealized = (market_data.last_price - position.average_price) * position.quantity;
                total_pnl += unrealized;
            }
        }
        
        total_pnl
    }

    fn calculate_performance_metrics(&mut self, final_capital: f64, daily_pnls: &[f64], max_drawdown: f64) -> BacktestResults {
        let total_return = (final_capital - self.config.initial_capital) / self.config.initial_capital;
        
        // Calculate volatility and Sharpe ratio
        let mean_daily_pnl = if !daily_pnls.is_empty() {
            daily_pnls.iter().sum::<f64>() / daily_pnls.len() as f64
        } else {
            0.0
        };
        
        let daily_variance = if daily_pnls.len() > 1 {
            daily_pnls.iter()
                .map(|&pnl| (pnl - mean_daily_pnl).powi(2))
                .sum::<f64>() / (daily_pnls.len() - 1) as f64
        } else {
            0.0
        };
        
        let daily_volatility = daily_variance.sqrt();
        let annualized_volatility = daily_volatility * (252.0_f64).sqrt();
        let annualized_return = mean_daily_pnl * 252.0;
        
        let sharpe_ratio = if daily_volatility > 0.0 {
            (annualized_return - 0.02) / annualized_volatility // Assuming 2% risk-free rate
        } else {
            0.0
        };
        
        // Calculate downside deviation for Sortino ratio
        let downside_returns: Vec<f64> = daily_pnls.iter()
            .filter(|&&pnl| pnl < 0.0)
            .cloned()
            .collect();
        
        let downside_variance = if !downside_returns.is_empty() {
            downside_returns.iter()
                .map(|&pnl| pnl.powi(2))
                .sum::<f64>() / downside_returns.len() as f64
        } else {
            daily_variance
        };
        
        let downside_deviation = downside_variance.sqrt() * (252.0_f64).sqrt();
        let sortino_ratio = if downside_deviation > 0.0 {
            (annualized_return - 0.02) / downside_deviation
        } else {
            0.0
        };
        
        // Calculate Calmar ratio
        let calmar_ratio = if max_drawdown > 0.0 {
            annualized_return / max_drawdown
        } else {
            0.0
        };
        
        // Calculate trade statistics
        let winning_trades = self.trade_history.iter().filter(|t| t.pnl > 0.0).count();
        let _losing_trades = self.trade_history.iter().filter(|t| t.pnl < 0.0).count();
        let win_rate = if !self.trade_history.is_empty() {
            winning_trades as f64 / self.trade_history.len() as f64
        } else {
            0.0
        };
        
        let gross_profit: f64 = self.trade_history.iter()
            .filter(|t| t.pnl > 0.0)
            .map(|t| t.pnl)
            .sum();
        let gross_loss: f64 = self.trade_history.iter()
            .filter(|t| t.pnl < 0.0)
            .map(|t| t.pnl.abs())
            .sum();
        
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else {
            0.0
        };
        
        let avg_trade_pnl = if !self.trade_history.is_empty() {
            self.trade_history.iter().map(|t| t.pnl).sum::<f64>() / self.trade_history.len() as f64
        } else {
            0.0
        };
        
        BacktestResults {
            total_return,
            sharpe_ratio,
            max_drawdown,
            total_trades: self.trade_history.len() as u32,
            win_rate,
            profit_factor,
            avg_trade_pnl,
            volatility: annualized_volatility,
            calmar_ratio,
            sortino_ratio,
            final_capital,
        }
    }

    pub fn get_trade_history(&self) -> &[Trade] {
        &self.trade_history
    }

    pub fn get_pnl_curve(&self) -> Vec<f64> {
        self.pnl_history.iter().cloned().collect()
    }

    pub fn get_drawdown_curve(&self) -> Vec<f64> {
        self.drawdown_history.iter().cloned().collect()
    }
}
