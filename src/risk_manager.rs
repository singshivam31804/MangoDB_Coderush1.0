
use crate::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RiskManager {
    config: RiskConfig,
    position_limits: PositionLimits,
    exposure_limits: ExposureLimits,
    var_calculator: VarCalculator,
    drawdown_monitor: DrawdownMonitor,
}

#[derive(Debug, Clone)]
struct RiskConfig {
    max_position_size: f64,
    max_daily_loss: f64,
    var_limit: f64,
    leverage_limit: f64,
    concentration_limit: f64,
}

#[derive(Debug, Clone)]
struct PositionLimits {
    max_gross_notional: f64,
    max_net_notional: f64,
    max_single_position: f64,
    max_sector_exposure: f64,
}

#[derive(Debug, Clone)]
struct ExposureLimits {
    gross_exposure_limit: f64,
    net_exposure_limit: f64,
    delta_limit: f64,
    gamma_limit: f64,
}

#[derive(Debug, Clone)]
struct VarCalculator {
    confidence_level: f64,
    time_horizon: f64,
    historical_returns: VecDeque<f64>,
    correlation_matrix: HashMap<String, HashMap<String, f64>>,
}

#[derive(Debug, Clone)]
struct DrawdownMonitor {
    max_allowed_drawdown: f64,
    current_drawdown: f64,
    peak_equity: f64,
    daily_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub var_95: f64,
    pub var_99: f64,
    pub expected_shortfall: f64,
    pub max_drawdown: f64,
    pub gross_exposure: f64,
    pub net_exposure: f64,
    pub leverage: f64,
    pub concentration_risk: f64,
    pub risk_score: f64,
}

impl RiskManager {
    pub fn new() -> Self {
        Self {
            config: RiskConfig {
                max_position_size: 1000000.0,
                max_daily_loss: 50000.0,
                var_limit: 100000.0,
                leverage_limit: 5.0,
                concentration_limit: 0.3,
            },
            position_limits: PositionLimits {
                max_gross_notional: 10000000.0,
                max_net_notional: 2000000.0,
                max_single_position: 500000.0,
                max_sector_exposure: 3000000.0,
            },
            exposure_limits: ExposureLimits {
                gross_exposure_limit: 15000000.0,
                net_exposure_limit: 3000000.0,
                delta_limit: 1000000.0,
                gamma_limit: 500000.0,
            },
            var_calculator: VarCalculator {
                confidence_level: 0.95,
                time_horizon: 1.0,
                historical_returns: VecDeque::new(),
                correlation_matrix: HashMap::new(),
            },
            drawdown_monitor: DrawdownMonitor {
                max_allowed_drawdown: 0.15,
                current_drawdown: 0.0,
                peak_equity: 1000000.0,
                daily_pnl: 0.0,
            },
        }
    }

    pub fn validate_order(&self, order: &Order, positions: &HashMap<String, Position>) -> bool {
        // Check position size limits
        if !self.check_position_limits(order, positions) {
            console_log!("Order rejected: Position limit exceeded");
            return false;
        }

        // Check exposure limits
        if !self.check_exposure_limits(order, positions) {
            console_log!("Order rejected: Exposure limit exceeded");
            return false;
        }

        // Check concentration limits
        if !self.check_concentration_limits(order, positions) {
            console_log!("Order rejected: Concentration limit exceeded");
            return false;
        }

        true
    }

    fn check_position_limits(&self, order: &Order, positions: &HashMap<String, Position>) -> bool {
        let notional = order.quantity * order.price;
        
        // Check single position limit
        if notional > self.position_limits.max_single_position {
            return false;
        }

        // Check if adding this order would exceed position limit for the symbol
        if let Some(position) = positions.get(&order.symbol) {
            let new_quantity = match order.side {
                OrderSide::Buy => position.quantity + order.quantity,
                OrderSide::Sell => position.quantity - order.quantity,
            };
            let new_notional = new_quantity.abs() * order.price;
            
            if new_notional > self.config.max_position_size {
                return false;
            }
        }

        true
    }

    fn check_exposure_limits(&self, order: &Order, positions: &HashMap<String, Position>) -> bool {
        let order_notional = order.quantity * order.price;
        let current_gross_exposure = self.calculate_gross_exposure(positions);
        
        // Check if adding this order would exceed gross exposure limit
        if current_gross_exposure + order_notional > self.exposure_limits.gross_exposure_limit {
            return false;
        }

        true
    }

    fn check_concentration_limits(&self, order: &Order, positions: &HashMap<String, Position>) -> bool {
        let order_notional = order.quantity * order.price;
        let total_portfolio_value = self.calculate_gross_exposure(positions);
        
        if total_portfolio_value > 0.0 {
            let concentration = order_notional / total_portfolio_value;
            if concentration > self.config.concentration_limit {
                return false;
            }
        }

        true
    }

    pub fn evaluate_risk(&mut self, positions: &HashMap<String, Position>, _quotes: &[Quote]) -> RiskMetrics {
        let gross_exposure = self.calculate_gross_exposure(positions);
        let net_exposure = self.calculate_net_exposure(positions);
        let leverage = self.calculate_leverage(positions);
        let concentration_risk = self.calculate_concentration_risk(positions);
        
        // Calculate VaR
        let var_95 = self.calculate_var(positions, 0.95);
        let var_99 = self.calculate_var(positions, 0.99);
        let expected_shortfall = self.calculate_expected_shortfall(positions, 0.95);
        
        // Calculate max drawdown
        let max_drawdown = self.drawdown_monitor.current_drawdown;
        
        // Calculate overall risk score
        let risk_score = self.calculate_risk_score(&RiskMetrics {
            var_95,
            var_99,
            expected_shortfall,
            max_drawdown,
            gross_exposure,
            net_exposure,
            leverage,
            concentration_risk,
            risk_score: 0.0, // Will be calculated
        });

        RiskMetrics {
            var_95,
            var_99,
            expected_shortfall,
            max_drawdown,
            gross_exposure,
            net_exposure,
            leverage,
            concentration_risk,
            risk_score,
        }
    }

    fn calculate_gross_exposure(&self, positions: &HashMap<String, Position>) -> f64 {
        positions.values()
            .map(|pos| pos.quantity.abs() * pos.average_price)
            .sum()
    }

    fn calculate_net_exposure(&self, positions: &HashMap<String, Position>) -> f64 {
        positions.values()
            .map(|pos| pos.quantity * pos.average_price)
            .sum()
    }

    fn calculate_leverage(&self, positions: &HashMap<String, Position>) -> f64 {
        let gross_exposure = self.calculate_gross_exposure(positions);
        let equity = self.drawdown_monitor.peak_equity + self.drawdown_monitor.daily_pnl;
        
        if equity > 0.0 {
            gross_exposure / equity
        } else {
            0.0
        }
    }

    fn calculate_concentration_risk(&self, positions: &HashMap<String, Position>) -> f64 {
        if positions.is_empty() {
            return 0.0;
        }

        let total_exposure = self.calculate_gross_exposure(positions);
        if total_exposure == 0.0 {
            return 0.0;
        }

        let largest_position = positions.values()
            .map(|pos| pos.quantity.abs() * pos.average_price)
            .fold(0.0f64, |a, b| a.max(b));

        largest_position / total_exposure
    }

    fn calculate_var(&mut self, positions: &HashMap<String, Position>, confidence_level: f64) -> f64 {
        // Simplified VaR calculation using historical simulation
        if self.var_calculator.historical_returns.len() < 30 {
            return 0.0;
        }

        let mut returns: Vec<f64> = self.var_calculator.historical_returns.iter().cloned().collect();
        returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((1.0 - confidence_level) * returns.len() as f64) as usize;
        let var_return = returns[index.min(returns.len() - 1)];

        let portfolio_value = self.calculate_gross_exposure(positions);
        portfolio_value * var_return.abs()
    }

    fn calculate_expected_shortfall(&mut self, positions: &HashMap<String, Position>, confidence_level: f64) -> f64 {
        if self.var_calculator.historical_returns.len() < 30 {
            return 0.0;
        }

        let mut returns: Vec<f64> = self.var_calculator.historical_returns.iter().cloned().collect();
        returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let cutoff_index = ((1.0 - confidence_level) * returns.len() as f64) as usize;
        let tail_returns: Vec<f64> = returns.iter().take(cutoff_index + 1).cloned().collect();

        if tail_returns.is_empty() {
            return 0.0;
        }

        let avg_tail_return = tail_returns.iter().sum::<f64>() / tail_returns.len() as f64;
        let portfolio_value = self.calculate_gross_exposure(positions);
        portfolio_value * avg_tail_return.abs()
    }

    pub fn update_returns(&mut self, portfolio_return: f64) {
        self.var_calculator.historical_returns.push_back(portfolio_return);
        
        if self.var_calculator.historical_returns.len() > 252 { // Keep 1 year of data
            self.var_calculator.historical_returns.pop_front();
        }
    }

    fn calculate_risk_score(&self, metrics: &RiskMetrics) -> f64 {
        let mut score = 0.0;

        // VaR component (0-25 points)
        score += (metrics.var_95 / self.config.var_limit * 25.0).min(25.0);

        // Drawdown component (0-20 points)
        score += (metrics.max_drawdown / self.drawdown_monitor.max_allowed_drawdown * 20.0).min(20.0);

        // Leverage component (0-20 points)
        score += (metrics.leverage / self.config.leverage_limit * 20.0).min(20.0);

        // Concentration component (0-15 points)
        score += (metrics.concentration_risk * 15.0).min(15.0);

        // Exposure component (0-10 points)
        score += (metrics.gross_exposure / self.exposure_limits.gross_exposure_limit * 10.0).min(10.0);

        score.min(100.0)
    }

    pub fn update_daily_pnl(&mut self, pnl_change: f64) {
        self.drawdown_monitor.daily_pnl += pnl_change;
    }

    pub fn is_risk_limit_breached(&self, metrics: &RiskMetrics) -> bool {
        metrics.risk_score > 80.0 || 
        metrics.max_drawdown > self.drawdown_monitor.max_allowed_drawdown ||
        metrics.leverage > self.config.leverage_limit ||
        metrics.var_95 > self.config.var_limit
    }
}
