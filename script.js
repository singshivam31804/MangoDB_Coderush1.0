
        import init, { HFTEngine } from './pkg/hft_market_maker.js';
        
        let engine = null;
        let priceChart = null;
        let orderBookChart = null;
        let volatilityChart = null;
        let latencyChart = null;
        let isRunning = false;
        let dataGenerationInterval = null;
        
        const priceData = [];
        const pnlData = [];
        const volatilityData = [];
        const latencyData = [];
        
        function log(message) {
            const logContainer = document.getElementById('logContainer');
            const timestamp = new Date().toLocaleTimeString();
            const entry = document.createElement('div');
            entry.className = 'log-entry';
            entry.innerHTML = `<span class="log-timestamp">[${timestamp}]</span>${message}`;
            logContainer.appendChild(entry);
            logContainer.scrollTop = logContainer.scrollHeight;
        }
        
        function updateStatus(status, text) {
            const indicator = document.getElementById('statusIndicator');
            const statusText = document.getElementById('statusText');
            
            indicator.className = `status-indicator status-${status}`;
            statusText.textContent = text;
        }
        
        function initCharts() {
            // Price & PnL Chart
            const priceCtx = document.getElementById('priceChart').getContext('2d');
            priceChart = new Chart(priceCtx, {
                type: 'line',
                data: {
                    labels: [],
                    datasets: [{
                        label: 'Price',
                        data: [],
                        borderColor: '#00ff00',
                        backgroundColor: 'rgba(0, 255, 0, 0.1)',
                        fill: false,
                        tension: 0.1
                    }, {
                        label: 'PnL',
                        data: [],
                        borderColor: '#ffaa00',
                        backgroundColor: 'rgba(255, 170, 0, 0.1)',
                        fill: false,
                        tension: 0.1,
                        yAxisID: 'y1'
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: { legend: { labels: { color: '#00ff00' } } },
                    scales: {
                        x: { ticks: { color: '#00ff00' }, grid: { color: '#333' } },
                        y: { ticks: { color: '#00ff00' }, grid: { color: '#333' } },
                        y1: { type: 'linear', display: true, position: 'right', ticks: { color: '#ffaa00' }, grid: { drawOnChartArea: false } }
                    }
                }
            });
            
            // Order Book Chart
            const obCtx = document.getElementById('orderBookChart').getContext('2d');
            orderBookChart = new Chart(obCtx, {
                type: 'bar',
                data: {
                    labels: ['Bids', 'Asks'],
                    datasets: [{
                        label: 'Volume',
                        data: [1000, 1200],
                        backgroundColor: ['rgba(0, 255, 0, 0.6)', 'rgba(255, 68, 68, 0.6)'],
                        borderColor: ['#00ff00', '#ff4444'],
                        borderWidth: 1
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: { legend: { labels: { color: '#00ff00' } } },
                    scales: {
                        x: { ticks: { color: '#00ff00' }, grid: { color: '#333' } },
                        y: { ticks: { color: '#00ff00' }, grid: { color: '#333' } }
                    }
                }
            });
            
            // Volatility Chart
            const volCtx = document.getElementById('volatilityChart').getContext('2d');
            volatilityChart = new Chart(volCtx, {
                type: 'line',
                data: {
                    labels: [],
                    datasets: [{
                        label: 'Volatility',
                        data: [],
                        borderColor: '#ff4444',
                        backgroundColor: 'rgba(255, 68, 68, 0.1)',
                        fill: true,
                        tension: 0.1
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: { legend: { labels: { color: '#00ff00' } } },
                    scales: {
                        x: { ticks: { color: '#00ff00' }, grid: { color: '#333' } },
                        y: { ticks: { color: '#00ff00' }, grid: { color: '#333' } }
                    }
                }
            });
            
            // Latency Chart
            const latCtx = document.getElementById('latencyChart').getContext('2d');
            latencyChart = new Chart(latCtx, {
                type: 'line',
                data: {
                    labels: [],
                    datasets: [{
                        label: 'Processing Latency (ms)',
                        data: [],
                        borderColor: '#00aaff',
                        backgroundColor: 'rgba(0, 170, 255, 0.1)',
                        fill: false,
                        tension: 0.1
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: { legend: { labels: { color: '#00ff00' } } },
                    scales: {
                        x: { ticks: { color: '#00ff00' }, grid: { color: '#333' } },
                        y: { ticks: { color: '#00ff00' }, grid: { color: '#333' } }
                    }
                }
            });
        }
        
        function generateSampleMarketData() {
            const basePrice = 18450 + Math.random() * 100;
            const volatility = parseFloat(document.getElementById('volatilityLevel').value);
            
            return {
                symbol: document.getElementById('symbol').value,
                timestamp: Date.now(),
                last_price: basePrice + (Math.random() - 0.5) * basePrice * volatility,
                bid_price: basePrice - Math.random() * 2,
                ask_price: basePrice + Math.random() * 2,
                bid_size: 100 + Math.random() * 500,
                ask_size: 100 + Math.random() * 500,
                volume: 1000 + Math.random() * 5000
            };
        }
        
        function updateCharts(marketData, result) {
            const timestamp = new Date().toLocaleTimeString();
            
            // Update price chart
            priceData.push(marketData.last_price);
            if (result && result.risk_metrics) {
                pnlData.push(result.risk_metrics.net_exposure);
            }
            
            if (priceData.length > 50) {
                priceData.shift();
                pnlData.shift();
            }
            
            priceChart.data.labels = Array.from({length: priceData.length}, (_, i) => i);
            priceChart.data.datasets[0].data = priceData;
            priceChart.data.datasets[1].data = pnlData;
            priceChart.update('none');
            
            // Update volatility chart
            if (result) {
                volatilityData.push(result.volatility);
                if (volatilityData.length > 50) volatilityData.shift();
                
                volatilityChart.data.labels = Array.from({length: volatilityData.length}, (_, i) => i);
                volatilityChart.data.datasets[0].data = volatilityData;
                volatilityChart.update('none');
            }
            
            // Update latency chart
            if (result && result.latency_stats) {
                const avgLatency = (result.latency_stats.avg_processing + result.latency_stats.avg_network) / 2;
                latencyData.push(avgLatency);
                if (latencyData.length > 50) latencyData.shift();
                
                latencyChart.data.labels = Array.from({length: latencyData.length}, (_, i) => i);
                latencyChart.data.datasets[0].data = latencyData;
                latencyChart.update('none');
            }
        }
        
        function updateMetrics(result) {
            if (!result) return;
            
            if (result.risk_metrics) {
                document.getElementById('totalPnL').textContent = `$${result.risk_metrics.net_exposure.toFixed(2)}`;
                document.getElementById('riskScore').textContent = result.risk_metrics.risk_score.toFixed(1);
                document.getElementById('leverage').textContent = result.risk_metrics.leverage.toFixed(2);
                document.getElementById('grossExposure').textContent = `$${result.risk_metrics.gross_exposure.toFixed(0)}`;
                document.getElementById('netExposure').textContent = `$${result.risk_metrics.net_exposure.toFixed(0)}`;
                document.getElementById('var95').textContent = `$${result.risk_metrics.var_95.toFixed(0)}`;
                document.getElementById('var99').textContent = `$${result.risk_metrics.var_99.toFixed(0)}`;
                document.getElementById('expectedShortfall').textContent = `$${result.risk_metrics.expected_shortfall.toFixed(0)}`;
            }
            
            document.getElementById('currentVolatility').textContent = (result.volatility * 100).toFixed(3) + '%';
            
            if (result.latency_stats) {
                document.getElementById('processingLatency').textContent = result.latency_stats.avg_processing.toFixed(3);
                document.getElementById('networkLatency').textContent = result.latency_stats.avg_network.toFixed(3);
                document.getElementById('totalLatency').textContent = (result.latency_stats.avg_processing + result.latency_stats.avg_network).toFixed(3);
            }
        }
        
        function startDataGeneration() {
            if (dataGenerationInterval) return;
            
            dataGenerationInterval = setInterval(() => {
                if (!isRunning || !engine) return;
                
                const marketData = generateSampleMarketData();
                const result = engine.process_market_data(marketData);
                
                updateCharts(marketData, result);
                updateMetrics(result);
                
                // Update order book display occasionally
                if (Math.random() < 0.1) {
                    updateOrderBookDisplay(marketData);
                }
                
            }, 100); // 10 updates per second
        }
        
        function stopDataGeneration() {
            if (dataGenerationInterval) {
                clearInterval(dataGenerationInterval);
                dataGenerationInterval = null;
            }
        }
        
        function updateOrderBookDisplay(marketData) {
            const tbody = document.getElementById('orderBookData');
            tbody.innerHTML = '';
            
            for (let i = 0; i < 5; i++) {
                const row = document.createElement('tr');
                row.className = 'bid-row';
                
                const bidQty = 100 + Math.random() * 200;
                const askQty = 100 + Math.random() * 200;
                const bidPrice = marketData.bid_price - i * 0.25;
                const askPrice = marketData.ask_price + i * 0.25;
                
                row.innerHTML = `
                    <td>${bidQty.toFixed(0)}</td>
                    <td>${bidPrice.toFixed(2)}</td>
                    <td>${askPrice.toFixed(2)}</td>
                    <td>${askQty.toFixed(0)}</td>
                `;
                
                tbody.appendChild(row);
            }
        }
        
        // Event Listeners
        document.getElementById('startEngine').addEventListener('click', () => {
            if (!engine) {
                log('Engine not initialized!');
                return;
            }
            
            isRunning = true;
            updateStatus('running', 'System Running');
            startDataGeneration();
            log('HFT Engine started successfully');
        });
        
        document.getElementById('stopEngine').addEventListener('click', () => {
            isRunning = false;
            updateStatus('stopped', 'System Stopped');
            stopDataGeneration();
            log('HFT Engine stopped');
        });
        
        document.getElementById('runBacktest').addEventListener('click', async () => {
            if (!engine) {
                log('Engine not initialized!');
                return;
            }
            
            log('Starting backtest...');
            updateStatus('warning', 'Running Backtest');
            
            // Generate historical data for backtest
            const historicalData = [];
            const dataPoints = parseInt(document.getElementById('dataPoints').value);
            
            for (let i = 0; i < dataPoints; i++) {
                historicalData.push(generateSampleMarketData());
            }
            
            try {
                const results = engine.run_backtest(historicalData);
                
                // Update backtest results display
                document.getElementById('totalReturn').textContent = (results.total_return * 100).toFixed(2) + '%';
                document.getElementById('sharpeRatio').textContent = results.sharpe_ratio.toFixed(3);
                document.getElementById('maxDrawdown').textContent = (results.max_drawdown * 100).toFixed(2) + '%';
                document.getElementById('profitFactor').textContent = results.profit_factor.toFixed(2);
                document.getElementById('calmarRatio').textContent = results.calmar_ratio.toFixed(3);
                document.getElementById('sortinoRatio').textContent = results.sortino_ratio.toFixed(3);
                document.getElementById('totalTrades').textContent = results.total_trades;
                document.getElementById('winRate').textContent = (results.win_rate * 100).toFixed(1) + '%';
                
                log(`Backtest completed: Return=${(results.total_return * 100).toFixed(2)}%, Sharpe=${results.sharpe_ratio.toFixed(3)}, MaxDD=${(results.max_drawdown * 100).toFixed(2)}%`);
                updateStatus('running', 'Backtest Complete');
                
            } catch (error) {
                log(`Backtest failed: ${error.message}`);
                updateStatus('warning', 'Backtest Failed');
            }
        });
        
        document.getElementById('resetSystem').addEventListener('click', () => {
            if (engine) {
                isRunning = false;
                stopDataGeneration();
                updateStatus('stopped', 'System Reset');
                log('System reset completed');
            }
        });
        
        document.getElementById('generateData').addEventListener('click', () => {
            log('Generating sample market data...');
            const dataPoints = parseInt(document.getElementById('dataPoints').value);
            
            for (let i = 0; i < Math.min(dataPoints, 100); i++) {
                const marketData = generateSampleMarketData();
                if (engine) {
                    const result = engine.process_market_data(marketData);
                    updateCharts(marketData, result);
                    updateMetrics(result);
                }
            }
            
            log(`Generated ${Math.min(dataPoints, 100)} data points`);
        });
        
        document.getElementById('benchmarkPerformance').addEventListener('click', () => {
            if (engine) {
                const benchmark = engine.benchmark_performance();
                log(`Performance benchmark: ${JSON.stringify(benchmark)}`);
            }
        });
        
        document.getElementById('simulateFPGA').addEventListener('click', () => {
            if (engine) {
                const optimizedLatency = engine.simulate_fpga_acceleration();
                log(`FPGA simulation: Latency optimized to ${optimizedLatency.toFixed(3)}ms`);
            }
        });
        
        document.getElementById('simulateGPU').addEventListener('click', () => {
            if (engine) {
                const optimizedLatency = engine.simulate_gpu_acceleration();
                log(`GPU simulation: Latency optimized to ${optimizedLatency.toFixed(3)}ms`);
            }
        });
        
        // Initialize everything
        const initApp = async () => {
            try {
                await init();
                engine = new HFTEngine();
                initCharts();
                log('HFT Engine initialized successfully');
                log('All systems ready');
                
                // Start with sample data
                setTimeout(() => {
                    document.getElementById('generateData').click();
                }, 1000);
                
            } catch (error) {
                log(`Initialization error: ${error.message}`);
            }
        };
        
        // Start the application
        initApp();
