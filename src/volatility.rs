use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct VolatilityModel {
    prices: VecDeque<f64>,
    returns: VecDeque<f64>,
    garch_params: GarchParameters,
    ewma_params: EwmaParameters,
    realized_volatility: f64,
    implied_volatility: f64,
    volatility_regime: VolatilityRegime,
}

#[derive(Debug, Clone)]
struct GarchParameters {
    omega: f64,    // Constant term
    alpha: f64,    // ARCH coefficient
    beta: f64,     // GARCH coefficient
    lambda: f64,   // Long-run variance
}

#[derive(Debug, Clone)]
struct EwmaParameters {
    decay_factor: f64,
    window_size: usize,
}

#[derive(Debug, Clone)]
enum VolatilityRegime {
    Low,
    Normal,
    High,
    Extreme,
}

impl VolatilityModel {
    pub fn new() -> Self {
        Self {
            prices: VecDeque::new(),
            returns: VecDeque::new(),
            garch_params: GarchParameters {
                omega: 0.000001,
                alpha: 0.1,
                beta: 0.85,
                lambda: 0.000016,
            },
            ewma_params: EwmaParameters {
                decay_factor: 0.94,
                window_size: 100,
            },
            realized_volatility: 0.0,
            implied_volatility: 0.0,
            volatility_regime: VolatilityRegime::Normal,
        }
    }

    pub fn update(&mut self, price: f64, _timestamp: f64) -> f64 {
        self.prices.push_back(price);

        // Maintain window size
        if self.prices.len() > 1000 {
            self.prices.pop_front();
        }

        // Calculate returns if we have at least 2 prices
        if self.prices.len() >= 2 {
            let prev_price = self.prices[self.prices.len() - 2];
            let return_value = (price / prev_price).ln();

            self.returns.push_back(return_value);

            if self.returns.len() > 500 {
                self.returns.pop_front();
            }
        }

        // Update volatility estimates
        if self.returns.len() >= 20 {
            self.realized_volatility = self.calculate_realized_volatility();
            self.update_volatility_regime();
        }

        self.realized_volatility
    }

    fn calculate_realized_volatility(&self) -> f64 {
        if self.returns.len() < 2 {
            return 0.0;
        }

        // Calculate EWMA volatility
        let ewma_vol = self.calculate_ewma_volatility();

        // Calculate GARCH volatility
        let garch_vol = self.calculate_garch_volatility();

        // Weighted average of different models
        let combined_vol = 0.6 * ewma_vol + 0.4 * garch_vol;

        // Annualize the volatility (assuming daily data)
        combined_vol * (252.0_f64).sqrt()
    }

    fn calculate_ewma_volatility(&self) -> f64 {
        if self.returns.is_empty() {
            return 0.0;
        }

        let mut weighted_var = 0.0;
        let mut weight_sum = 0.0;
        let decay = self.ewma_params.decay_factor;

        for (i, &return_val) in self.returns.iter().rev().enumerate() {
            let weight = decay.powi(i as i32);
            weighted_var += weight * return_val.powi(2);
            weight_sum += weight;

            if i >= self.ewma_params.window_size {
                break;
            }
        }

        if weight_sum > 0.0 {
            (weighted_var / weight_sum).sqrt()
        } else {
            0.0
        }
    }

    fn calculate_garch_volatility(&self) -> f64 {
        if self.returns.len() < 10 {
            return 0.0;
        }

        // Simplified GARCH(1,1) calculation
        let recent_returns: Vec<f64> = self.returns.iter().rev().take(10).cloned().collect();

        // Calculate sample variance for initialization
        let mean_return = recent_returns.iter().sum::<f64>() / recent_returns.len() as f64;
        let sample_variance = recent_returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / (recent_returns.len() - 1) as f64;

        let mut conditional_variance = sample_variance;

        // Update conditional variance using GARCH(1,1)
        for &return_val in recent_returns.iter().rev() {
            conditional_variance = self.garch_params.omega +
                                 self.garch_params.alpha * return_val.powi(2) +
                                 self.garch_params.beta * conditional_variance;
        }

        conditional_variance.sqrt()
    }

    fn update_volatility_regime(&mut self) {
        let vol = self.realized_volatility;

        self.volatility_regime = if vol < 0.1 {
            VolatilityRegime::Low
        } else if vol < 0.2 {
            VolatilityRegime::Normal
        } else if vol < 0.4 {
            VolatilityRegime::High
        } else {
            VolatilityRegime::Extreme
        };
    }

    pub fn get_volatility(&self) -> f64 {
        self.realized_volatility
    }

    pub fn get_volatility_percentile(&self) -> f64 {
        if self.returns.len() < 50 {
            return 0.5; // Default to median
        }

        // Calculate historical volatilities
        let mut historical_vols = Vec::new();
        let window_size = 20;

        for i in window_size..self.returns.len() {
            let window_returns: Vec<f64> = self.returns.iter()
                .skip(i - window_size)
                .take(window_size)
                .cloned()
                .collect();

            let vol = self.calculate_window_volatility(&window_returns);
            historical_vols.push(vol);
        }

        if historical_vols.is_empty() {
            return 0.5;
        }

        // Sort and find percentile
        historical_vols.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let current_vol = self.realized_volatility;

        let below_current = historical_vols.iter()
            .filter(|&&vol| vol <= current_vol)
            .count();

        below_current as f64 / historical_vols.len() as f64
    }

    fn calculate_window_volatility(&self, returns: &[f64]) -> f64 {
        if returns.len() < 2 {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;

        variance.sqrt() * (252.0_f64).sqrt()
    }

    pub fn forecast_volatility(&self, horizon_days: usize) -> f64 {
        if self.returns.is_empty() {
            return self.realized_volatility;
        }

        // Simple GARCH forecasting
        let current_vol = self.realized_volatility;
        let long_run_vol = self.garch_params.lambda.sqrt();

        // Mean reversion forecast
        let persistence = self.garch_params.alpha + self.garch_params.beta;
        let decay_factor = persistence.powi(horizon_days as i32);

        let forecast = long_run_vol + decay_factor * (current_vol - long_run_vol);
        forecast.max(0.01) // Minimum volatility floor
    }

    pub fn get_volatility_clustering_score(&self) -> f64 {
        if self.returns.len() < 10 {
            return 0.0;
        }

        // Calculate volatility clustering using absolute returns
        let abs_returns: Vec<f64> = self.returns.iter()
            .map(|&r| r.abs())
            .collect();

        if abs_returns.len() < 5 {
            return 0.0;
        }

        // Calculate autocorrelation of absolute returns
        let mean_abs = abs_returns.iter().sum::<f64>() / abs_returns.len() as f64;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 1..abs_returns.len() {
            numerator += (abs_returns[i] - mean_abs) * (abs_returns[i-1] - mean_abs);
            denominator += (abs_returns[i] - mean_abs).powi(2);
        }

        if denominator > 0.0 {
            (numerator / denominator).abs()
        } else {
            0.0
        }
    }

    pub fn is_high_volatility_regime(&self) -> bool {
        matches!(self.volatility_regime, VolatilityRegime::High | VolatilityRegime::Extreme)
    }

    pub fn get_vol_regime_string(&self) -> String {
        match self.volatility_regime {
            VolatilityRegime::Low => "Low".to_string(),
            VolatilityRegime::Normal => "Normal".to_string(),
            VolatilityRegime::High => "High".to_string(),
            VolatilityRegime::Extreme => "Extreme".to_string(),
        }
    }
}
