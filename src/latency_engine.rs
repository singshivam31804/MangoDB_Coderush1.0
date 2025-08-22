
use crate::*;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct LatencyEngine {
    latency_stats: LatencyStatistics,
    execution_stats: ExecutionStatistics,
    network_monitor: NetworkMonitor,
    optimization_config: OptimizationConfig,
}

#[derive(Debug, Clone)]
struct LatencyStatistics {
    processing_latencies: VecDeque<f64>,
    tick_to_trade_latencies: VecDeque<f64>,
    order_book_update_latencies: VecDeque<f64>,
    quote_generation_latencies: VecDeque<f64>,
    window_size: usize,
}

#[derive(Debug, Clone)]
struct ExecutionStatistics {
    order_execution_latencies: VecDeque<f64>,
    fill_latencies: VecDeque<f64>,
    cancel_latencies: VecDeque<f64>,
    modify_latencies: VecDeque<f64>,
}

#[derive(Debug, Clone)]
struct NetworkMonitor {
    round_trip_times: VecDeque<f64>,
    packet_loss_rate: f64,
    bandwidth_utilization: f64,
    jitter: f64,
}

#[derive(Debug, Clone)]
struct OptimizationConfig {
    target_latency_percentile: f64,
    target_latency_threshold: f64,
    cache_warmup_enabled: bool,
    prediction_enabled: bool,
    async_processing_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub avg_processing: f64,
    pub p99_processing: f64,
    pub avg_network: f64,
    pub p99_network: f64,
    pub tick_to_trade: f64,
    pub order_book_update: f64,
    pub quote_generation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmark {
    pub processing_throughput: f64,
    pub latency_percentiles: LatencyPercentiles,
    pub optimization_score: f64,
    pub recommended_optimizations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub p99_9: f64,
}

impl OptimizationConfig {
    fn new() -> Self {
        Self {
            target_latency_percentile: 0.99,
            target_latency_threshold: 1.0, // 1ms threshold
            cache_warmup_enabled: true,
            prediction_enabled: true,
            async_processing_enabled: true,
        }
    }
}

impl LatencyStatistics {
    fn new() -> Self {
        Self {
            processing_latencies: VecDeque::new(),
            tick_to_trade_latencies: VecDeque::new(),
            order_book_update_latencies: VecDeque::new(),
            quote_generation_latencies: VecDeque::new(),
            window_size: 1000,
        }
    }
}

impl ExecutionStatistics {
    fn new() -> Self {
        Self {
            order_execution_latencies: VecDeque::new(),
            fill_latencies: VecDeque::new(),
            cancel_latencies: VecDeque::new(),
            modify_latencies: VecDeque::new(),
        }
    }
}

impl NetworkMonitor {
    fn new() -> Self {
        Self {
            round_trip_times: VecDeque::new(),
            packet_loss_rate: 0.0,
            bandwidth_utilization: 0.0,
            jitter: 0.0,
        }
    }
}

impl LatencyEngine {
    pub fn new() -> Self {
        Self {
            latency_stats: LatencyStatistics::new(),
            execution_stats: ExecutionStatistics::new(),
            network_monitor: NetworkMonitor::new(),
            optimization_config: OptimizationConfig::new(),
        }
    }

    pub fn record_latency(&mut self, latency: f64) {
        self.record_processing_latency(latency);
    }

    pub fn record_processing_latency(&mut self, latency: f64) {
        self.latency_stats.processing_latencies.push_back(latency);
        
        if self.latency_stats.processing_latencies.len() > self.latency_stats.window_size {
            self.latency_stats.processing_latencies.pop_front();
        }

        // Check if optimization is needed
        if self.should_trigger_optimization() {
            self.trigger_latency_optimization();
        }
    }

    pub fn record_tick_to_trade_latency(&mut self, latency: f64) {
        self.latency_stats.tick_to_trade_latencies.push_back(latency);
        
        if self.latency_stats.tick_to_trade_latencies.len() > self.latency_stats.window_size {
            self.latency_stats.tick_to_trade_latencies.pop_front();
        }
    }

    pub fn record_order_book_update_latency(&mut self, latency: f64) {
        self.latency_stats.order_book_update_latencies.push_back(latency);
        
        if self.latency_stats.order_book_update_latencies.len() > self.latency_stats.window_size {
            self.latency_stats.order_book_update_latencies.pop_front();
        }
    }

    pub fn record_quote_generation_latency(&mut self, latency: f64) {
        self.latency_stats.quote_generation_latencies.push_back(latency);
        
        if self.latency_stats.quote_generation_latencies.len() > self.latency_stats.window_size {
            self.latency_stats.quote_generation_latencies.pop_front();
        }
    }

    pub fn record_execution_latency(&mut self, latency: f64, operation: &str) {
        match operation {
            "execution" => {
                self.execution_stats.order_execution_latencies.push_back(latency);
                if self.execution_stats.order_execution_latencies.len() > 1000 {
                    self.execution_stats.order_execution_latencies.pop_front();
                }
            },
            "fill" => {
                self.execution_stats.fill_latencies.push_back(latency);
                if self.execution_stats.fill_latencies.len() > 1000 {
                    self.execution_stats.fill_latencies.pop_front();
                }
            },
            "cancel" => {
                self.execution_stats.cancel_latencies.push_back(latency);
                if self.execution_stats.cancel_latencies.len() > 1000 {
                    self.execution_stats.cancel_latencies.pop_front();
                }
            },
            "modify" => {
                self.execution_stats.modify_latencies.push_back(latency);
                if self.execution_stats.modify_latencies.len() > 1000 {
                    self.execution_stats.modify_latencies.pop_front();
                }
            },
            _ => {}
        }
    }

    pub fn update_network_stats(&mut self, rtt: f64, packet_loss: f64, bandwidth: f64) {
        self.network_monitor.round_trip_times.push_back(rtt);
        self.network_monitor.packet_loss_rate = packet_loss;
        self.network_monitor.bandwidth_utilization = bandwidth;
        
        if self.network_monitor.round_trip_times.len() > 100 {
            self.network_monitor.round_trip_times.pop_front();
        }

        // Update jitter calculation
        self.update_jitter();
    }

    fn update_jitter(&mut self) {
        if self.network_monitor.round_trip_times.len() < 2 {
            return;
        }

        let rtts: Vec<f64> = self.network_monitor.round_trip_times.iter().cloned().collect();
        let mean_rtt = rtts.iter().sum::<f64>() / rtts.len() as f64;
        
        let variance = rtts.iter()
            .map(|rtt| (rtt - mean_rtt).powi(2))
            .sum::<f64>() / (rtts.len() - 1) as f64;
        
        self.network_monitor.jitter = variance.sqrt();
    }

    pub fn get_stats(&self) -> LatencyStats {
        LatencyStats {
            avg_processing: self.calculate_average_latency(&self.latency_stats.processing_latencies),
            p99_processing: self.calculate_percentile(&self.latency_stats.processing_latencies, 0.99),
            avg_network: self.calculate_average_latency(&self.network_monitor.round_trip_times),
            p99_network: self.calculate_percentile(&self.network_monitor.round_trip_times, 0.99),
            tick_to_trade: self.calculate_average_latency(&self.latency_stats.tick_to_trade_latencies),
            order_book_update: self.calculate_average_latency(&self.latency_stats.order_book_update_latencies),
            quote_generation: self.calculate_average_latency(&self.latency_stats.quote_generation_latencies),
        }
    }

    fn calculate_average_latency(&self, latencies: &VecDeque<f64>) -> f64 {
        if latencies.is_empty() {
            return 0.0;
        }
        latencies.iter().sum::<f64>() / latencies.len() as f64
    }

    fn calculate_percentile(&self, latencies: &VecDeque<f64>, percentile: f64) -> f64 {
        if latencies.is_empty() {
            return 0.0;
        }

        let mut sorted: Vec<f64> = latencies.iter().cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = (percentile * (sorted.len() - 1) as f64) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    fn should_trigger_optimization(&self) -> bool {
        if self.latency_stats.processing_latencies.len() < 100 {
            return false;
        }

        let p99_latency = self.calculate_percentile(&self.latency_stats.processing_latencies, 0.99);
        p99_latency > self.optimization_config.target_latency_threshold
    }

    pub fn benchmark_processing_pipeline(&mut self) -> PerformanceBenchmark {
        console_log!("Running performance benchmark...");
        
        // Simulate benchmark processing
        let start_time = now();
        
        // Simulate various processing operations
        for i in 0..1000 {
            let _operation_start = now();
            
            // Simulate processing delay
            let simulated_latency = 0.1 + (i as f64 % 10.0) * 0.05;
            self.record_processing_latency(simulated_latency);
            
            // Simulate different operation types
            match i % 4 {
                0 => self.record_tick_to_trade_latency(simulated_latency * 1.2),
                1 => self.record_order_book_update_latency(simulated_latency * 0.8),
                2 => self.record_quote_generation_latency(simulated_latency * 1.1),
                _ => self.record_execution_latency(simulated_latency * 0.9, "execution"),
            }
        }
        
        let total_benchmark_time = now() - start_time;
        let throughput = 1000.0 / total_benchmark_time * 1000.0; // operations per second
        
        let percentiles = LatencyPercentiles {
            p50: self.calculate_percentile(&self.latency_stats.processing_latencies, 0.50),
            p95: self.calculate_percentile(&self.latency_stats.processing_latencies, 0.95),
            p99: self.calculate_percentile(&self.latency_stats.processing_latencies, 0.99),
            p99_9: self.calculate_percentile(&self.latency_stats.processing_latencies, 0.999),
        };
        
        let optimization_score = self.calculate_optimization_score();
        let recommendations = self.generate_optimization_recommendations();
        
        console_log!("Benchmark completed: {:.0} ops/sec, P99: {:.3}ms", throughput, percentiles.p99);
        
        PerformanceBenchmark {
            processing_throughput: throughput,
            latency_percentiles: percentiles,
            optimization_score,
            recommended_optimizations: recommendations,
        }
    }

    fn calculate_optimization_score(&self) -> f64 {
        let mut score = 100.0;
        
        // Penalize high average latency
        let avg_latency = self.calculate_average_latency(&self.latency_stats.processing_latencies);
        if avg_latency > 1.0 {
            score -= (avg_latency - 1.0) * 10.0;
        }
        
        // Penalize high P99 latency
        let p99_latency = self.calculate_percentile(&self.latency_stats.processing_latencies, 0.99);
        if p99_latency > 5.0 {
            score -= (p99_latency - 5.0) * 5.0;
        }
        
        // Penalize high jitter
        if self.network_monitor.jitter > 0.5 {
            score -= (self.network_monitor.jitter - 0.5) * 20.0;
        }
        
        // Penalize packet loss
        score -= self.network_monitor.packet_loss_rate * 50.0;
        
        score.max(0.0).min(100.0)
    }

    fn generate_optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        let avg_latency = self.calculate_average_latency(&self.latency_stats.processing_latencies);
        let p99_latency = self.calculate_percentile(&self.latency_stats.processing_latencies, 0.99);
        
        if avg_latency > 2.0 {
            recommendations.push("Consider CPU frequency scaling optimization".to_string());
            recommendations.push("Enable cache warmup procedures".to_string());
        }
        
        if p99_latency > 10.0 {
            recommendations.push("Implement predictive processing for tail latency reduction".to_string());
            recommendations.push("Consider FPGA acceleration for critical path".to_string());
        }
        
        if self.network_monitor.jitter > 1.0 {
            recommendations.push("Optimize network buffer sizes".to_string());
            recommendations.push("Consider dedicated network interface".to_string());
        }
        
        if self.network_monitor.packet_loss_rate > 0.01 {
            recommendations.push("Implement packet loss recovery mechanisms".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("System performance is optimal".to_string());
        }
        
        recommendations
    }

    fn trigger_latency_optimization(&mut self) {
        console_log!("Triggering latency optimization due to high latency");
        
        // Implement various optimization strategies
        if self.optimization_config.cache_warmup_enabled {
            self.warm_up_caches();
        }
        
        if self.optimization_config.prediction_enabled {
            self.enable_predictive_processing();
        }
        
        if self.optimization_config.async_processing_enabled {
            self.optimize_async_processing();
        }
    }

    fn warm_up_caches(&self) {
        // Simulate cache warmup
        console_log!("Warming up caches for latency optimization");
    }

    fn enable_predictive_processing(&self) {
        // Simulate predictive processing enablement
        console_log!("Enabling predictive processing for latency optimization");
    }

    fn optimize_async_processing(&self) {
        // Simulate async processing optimization
        console_log!("Optimizing async processing for latency reduction");
    }

    pub fn simulate_fpga_acceleration(&mut self) -> f64 {
        // Simulate FPGA acceleration benefits
        let baseline_latency = self.calculate_average_latency(&self.latency_stats.processing_latencies);
        let fpga_acceleration_factor = 0.1; // 90% latency reduction
        
        let optimized_latency = baseline_latency * fpga_acceleration_factor;
        console_log!("FPGA acceleration: {} -> {} ms", baseline_latency, optimized_latency);
        
        // Record optimized latencies
        for _ in 0..100 {
            self.record_processing_latency(optimized_latency + (now() % 1.0) * 0.01);
        }
        
        optimized_latency
    }

    pub fn simulate_gpu_acceleration(&mut self) -> f64 {
        // Simulate GPU acceleration for parallel processing
        let baseline_latency = self.calculate_average_latency(&self.latency_stats.processing_latencies);
        let gpu_acceleration_factor = 0.3; // 70% latency reduction for parallel workloads
        
        let optimized_latency = baseline_latency * gpu_acceleration_factor;
        console_log!("GPU acceleration: {} -> {} ms", baseline_latency, optimized_latency);
        
        // Record optimized latencies
        for _ in 0..100 {
            self.record_processing_latency(optimized_latency + (now() % 1.0) * 0.02);
        }
        
        optimized_latency
    }

    pub fn get_real_time_latency_metrics(&self) -> LatencyStats {
        // Get the most recent latency measurements
        let recent_window = 50; // Last 50 measurements
        
        let recent_processing: VecDeque<f64> = self.latency_stats.processing_latencies
            .iter()
            .rev()
            .take(recent_window)
            .cloned()
            .collect();
            
        let recent_network: VecDeque<f64> = self.network_monitor.round_trip_times
            .iter()
            .rev()
            .take(recent_window)
            .cloned()
            .collect();
        
        LatencyStats {
            avg_processing: self.calculate_average_latency(&recent_processing),
            p99_processing: self.calculate_percentile(&recent_processing, 0.99),
            avg_network: self.calculate_average_latency(&recent_network),
            p99_network: self.calculate_percentile(&recent_network, 0.99),
            tick_to_trade: self.calculate_average_latency(&self.latency_stats.tick_to_trade_latencies),
            order_book_update: self.calculate_average_latency(&self.latency_stats.order_book_update_latencies),
            quote_generation: self.calculate_average_latency(&self.latency_stats.quote_generation_latencies),
        }
    }
}
