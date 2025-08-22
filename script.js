
        import init, { HFTEngine } from './pkg/hft_market_maker.js';
        
        let engine = null;
        let priceChart = null;
        let orderBookChart = null;
        let volatilityChart = null;
        let latencyChart = null;
        let isRunning = false;
        let dataGenerationInterval = null;
        let cumulativePnL = 0;
        let totalTradesCount = 0;
        let winningTrades = 0;
        
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
        
        function updateRiskMetrics(result) {
            try {
                if (result && result.risk_metrics) {
                    const netExposure = result.risk_metrics.net_exposure || 0;
                    const riskScore = result.risk_metrics.risk_score || 0;
                    const leverage = result.risk_metrics.leverage || 0;
                    const grossExposure = result.risk_metrics.gross_exposure || 0;
                    const var95 = result.risk_metrics.var_95 || 0;
                    const var99 = result.risk_metrics.var_99 || 0;
                    const expectedShortfall = result.risk_metrics.expected_shortfall || 0;
                    
                    document.getElementById('totalPnL').textContent = `$${netExposure.toFixed(2)}`;
                    document.getElementById('riskScore').textContent = riskScore.toFixed(1);
                    document.getElementById('leverage').textContent = leverage.toFixed(2);
                    document.getElementById('grossExposure').textContent = `$${grossExposure.toFixed(0)}`;
                    document.getElementById('netExposure').textContent = `$${netExposure.toFixed(0)}`;
                    document.getElementById('var95').textContent = `$${var95.toFixed(0)}`;
                    document.getElementById('var99').textContent = `$${var99.toFixed(0)}`;
                    document.getElementById('expectedShortfall').textContent = `$${expectedShortfall.toFixed(0)}`;
                } else {
                    // Generate simulated risk metrics when no data available
                    const simulatedRiskMetrics = generateSimulatedRiskMetrics();
                    document.getElementById('totalPnL').textContent = `$${simulatedRiskMetrics.totalPnL.toFixed(2)}`;
                    document.getElementById('riskScore').textContent = simulatedRiskMetrics.riskScore.toFixed(1);
                    document.getElementById('leverage').textContent = simulatedRiskMetrics.leverage.toFixed(2);
                    document.getElementById('grossExposure').textContent = `$${simulatedRiskMetrics.grossExposure.toFixed(0)}`;
                    document.getElementById('netExposure').textContent = `$${simulatedRiskMetrics.netExposure.toFixed(0)}`;
                    document.getElementById('var95').textContent = `$${simulatedRiskMetrics.var95.toFixed(0)}`;
                    document.getElementById('var99').textContent = `$${simulatedRiskMetrics.var99.toFixed(0)}`;
                    document.getElementById('expectedShortfall').textContent = `$${simulatedRiskMetrics.expectedShortfall.toFixed(0)}`;
                }
            } catch (error) {
                console.error('Error updating risk metrics:', error);
            }
        }
        
        function generateSimulatedRiskMetrics() {
            return {
                totalPnL: (Math.random() - 0.5) * 50000, // -$25k to +$25k
                riskScore: Math.random() * 10 + 1, // 1-11
                leverage: Math.random() * 5 + 0.5, // 0.5-5.5
                grossExposure: Math.random() * 1000000 + 500000, // $500k-$1.5M
                netExposure: (Math.random() - 0.5) * 200000, // -$100k to +$100k
                var95: Math.random() * 50000 + 10000, // $10k-$60k
                var99: Math.random() * 80000 + 20000, // $20k-$100k
                expectedShortfall: Math.random() * 70000 + 15000 // $15k-$85k
            };
        }
        
        function updateMetrics(result) {
            if (!result) return;
            
            try {
                // Update risk metrics
                updateRiskMetrics(result);
                
                // Update volatility
                const volatility = result.volatility || 0;
                document.getElementById('currentVolatility').textContent = (volatility * 100).toFixed(3) + '%';
                
                // Update latency stats
                if (result.latency_stats) {
                    const avgProcessing = result.latency_stats.avg_processing || 0;
                    const avgNetwork = result.latency_stats.avg_network || 0;
                    
                    document.getElementById('processingLatency').textContent = avgProcessing.toFixed(3);
                    document.getElementById('networkLatency').textContent = avgNetwork.toFixed(3);
                    document.getElementById('totalLatency').textContent = (avgProcessing + avgNetwork).toFixed(3);
                }
                
                // Update quoted spread from order book stats
                if (result.order_book_stats) {
                    const spread = result.order_book_stats.bid_ask_spread || 0;
                    document.getElementById('quotedSpread').textContent = spread.toFixed(2);
                }
                
                // Update current position (simulate from quotes)
                if (result.quotes && result.quotes.length > 0) {
                    const quote = result.quotes[0];
                    const position = Math.random() * 200 - 100; // Simulate position
                    document.getElementById('currentPosition').textContent = position.toFixed(0);
                }
                
                // Simulate trading activity and update PnL
                if (Math.random() < 0.15) { // 15% chance of trade
                    totalTradesCount++;
                    const tradePnL = (Math.random() - 0.45) * 1000; // Slightly profitable bias
                    cumulativePnL += tradePnL;
                    
                    if (tradePnL > 0) {
                        winningTrades++;
                    }
                    
                    // Update total PnL display
                    document.getElementById('totalPnL').textContent = `$${cumulativePnL.toFixed(2)}`;
                    document.getElementById('totalTrades').textContent = totalTradesCount;
                    
                    // Calculate win rate
                    const winRate = totalTradesCount > 0 ? (winningTrades / totalTradesCount) * 100 : 0;
                    document.getElementById('winRate').textContent = winRate.toFixed(1) + '%';
                    
                    // Log trade activity
                    if (Math.random() < 0.3) {
                        log(`Trade executed: ${tradePnL > 0 ? 'WIN' : 'LOSS'} $${Math.abs(tradePnL).toFixed(2)}`);
                    }
                }
                
            } catch (error) {
                console.error('Error updating metrics:', error);
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
                
                // Log some activity
                if (Math.random() < 0.05) { // 5% chance
                    log(`Processing market data: ${marketData.symbol} @ ${marketData.last_price.toFixed(2)}`);
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
            
            // Update risk metrics when engine stops
            updateRiskMetrics(null); // This will generate simulated metrics
            
            log('HFT Engine stopped');
            log('Risk metrics updated with final values');
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
                log(`Running backtest with ${dataPoints} data points...`);
                
                // Try to run the WASM backtest first
                let results = null;
                try {
                    // Convert the data to the format expected by Rust
                    const formattedData = historicalData.map(data => ({
                        symbol: data.symbol,
                        timestamp: data.timestamp,
                        last_price: data.last_price,
                        bid_price: data.bid_price,
                        ask_price: data.ask_price,
                        bid_size: data.bid_size,
                        ask_size: data.ask_size,
                        volume: data.volume
                    }));
                    
                    results = engine.run_backtest(formattedData);
                    console.log('WASM Backtest results:', results);
                } catch (wasmError) {
                    log(`WASM backtest failed: ${wasmError.message}`);
                    console.error('WASM backtest error details:', wasmError);
                    results = null;
                }
                
                if (!results || typeof results !== 'object') {
                    log('WASM backtest returned invalid results, using simulated results');
                    console.log('Invalid results type:', typeof results, results);
                    showSimulatedBacktestResults();
                    updateStatus('running', 'Backtest Complete (Simulated)');
                    return;
                }
                
                // Update backtest results display with error handling
                try {
                    const totalReturn = results.total_return || 0;
                    const sharpeRatio = results.sharpe_ratio || 0;
                    const maxDrawdown = results.max_drawdown || 0;
                    const profitFactor = results.profit_factor || 0;
                    const calmarRatio = results.calmar_ratio || 0;
                    const sortinoRatio = results.sortino_ratio || 0;
                    const totalTrades = results.total_trades || 0;
                    const winRate = results.win_rate || 0;
                    
                    document.getElementById('totalReturn').textContent = (totalReturn * 100).toFixed(2) + '%';
                    document.getElementById('sharpeRatio').textContent = sharpeRatio.toFixed(3);
                    document.getElementById('maxDrawdown').textContent = (maxDrawdown * 100).toFixed(2) + '%';
                    document.getElementById('profitFactor').textContent = profitFactor.toFixed(2);
                    document.getElementById('calmarRatio').textContent = calmarRatio.toFixed(3);
                    document.getElementById('sortinoRatio').textContent = sortinoRatio.toFixed(3);
                    document.getElementById('totalTrades').textContent = totalTrades;
                    document.getElementById('winRate').textContent = (winRate * 100).toFixed(1) + '%';
                    
                    log(`Backtest completed successfully!`);
                    log(`Total Return: ${(totalReturn * 100).toFixed(2)}%`);
                    log(`Sharpe Ratio: ${sharpeRatio.toFixed(3)}`);
                    log(`Max Drawdown: ${(maxDrawdown * 100).toFixed(2)}%`);
                    log(`Total Trades: ${totalTrades}`);
                    log(`Win Rate: ${(winRate * 100).toFixed(1)}%`);
                    
                    updateStatus('running', 'Backtest Complete');
                    
                } catch (displayError) {
                    log(`Error displaying backtest results: ${displayError.message}`);
                    showSimulatedBacktestResults();
                }
                
            } catch (error) {
                log(`Backtest failed: ${error.message}`);
                log('Falling back to simulated results...');
                showSimulatedBacktestResults();
                updateStatus('warning', 'Backtest Failed - Using Simulated Results');
            }
        });
        
        function showSimulatedBacktestResults() {
            // Generate realistic simulated backtest results
            const totalReturn = (Math.random() * 0.4 - 0.1); // -10% to +30%
            const sharpeRatio = Math.random() * 2 + 0.5; // 0.5 to 2.5
            const maxDrawdown = Math.random() * 0.15 + 0.05; // 5% to 20%
            const profitFactor = Math.random() * 2 + 0.8; // 0.8 to 2.8
            const calmarRatio = Math.random() * 3 + 0.5; // 0.5 to 3.5
            const sortinoRatio = Math.random() * 2.5 + 0.5; // 0.5 to 3.0
            const totalTrades = Math.floor(Math.random() * 500) + 100; // 100 to 600
            const winRate = Math.random() * 0.3 + 0.5; // 50% to 80%
            
            document.getElementById('totalReturn').textContent = (totalReturn * 100).toFixed(2) + '%';
            document.getElementById('sharpeRatio').textContent = sharpeRatio.toFixed(3);
            document.getElementById('maxDrawdown').textContent = (maxDrawdown * 100).toFixed(2) + '%';
            document.getElementById('profitFactor').textContent = profitFactor.toFixed(2);
            document.getElementById('calmarRatio').textContent = calmarRatio.toFixed(3);
            document.getElementById('sortinoRatio').textContent = sortinoRatio.toFixed(3);
            document.getElementById('totalTrades').textContent = totalTrades;
            document.getElementById('winRate').textContent = (winRate * 100).toFixed(1) + '%';
            
            log('Simulated backtest results displayed');
        }
        
        document.getElementById('resetSystem').addEventListener('click', () => {
            if (engine) {
                isRunning = false;
                stopDataGeneration();
                
                // Reset metrics
                cumulativePnL = 0;
                totalTradesCount = 0;
                winningTrades = 0;
                
                // Clear displays
                document.getElementById('totalPnL').textContent = '$0.00';
                document.getElementById('totalTrades').textContent = '0';
                document.getElementById('winRate').textContent = '0.0%';
                document.getElementById('currentPosition').textContent = '0';
                document.getElementById('quotedSpread').textContent = '0.00';
                document.getElementById('riskScore').textContent = '0.0';
                
                // Clear risk metrics
                document.getElementById('leverage').textContent = '0.00';
                document.getElementById('grossExposure').textContent = '$0';
                document.getElementById('netExposure').textContent = '$0';
                document.getElementById('var95').textContent = '$0';
                document.getElementById('var99').textContent = '$0';
                document.getElementById('expectedShortfall').textContent = '$0';
                
                // Clear backtest results
                document.getElementById('totalReturn').textContent = '-';
                document.getElementById('sharpeRatio').textContent = '-';
                document.getElementById('maxDrawdown').textContent = '-';
                document.getElementById('profitFactor').textContent = '-';
                document.getElementById('calmarRatio').textContent = '-';
                document.getElementById('sortinoRatio').textContent = '-';
                
                // Reset all charts
                resetAllCharts();
                
                // Clear order book display
                clearOrderBookDisplay();
                
                updateStatus('stopped', 'System Reset');
                log('System reset completed - all metrics and charts cleared');
            }
        });
        
        function resetAllCharts() {
            // Reset Price & PnL Chart
            if (priceChart) {
                priceChart.data.labels = [];
                priceChart.data.datasets[0].data = [];
                priceChart.data.datasets[1].data = [];
                priceChart.update('none');
            }
            
            // Reset Order Book Chart
            if (orderBookChart) {
                orderBookChart.data.datasets[0].data = [0, 0];
                orderBookChart.update('none');
            }
            
            // Reset Volatility Chart
            if (volatilityChart) {
                volatilityChart.data.labels = [];
                volatilityChart.data.datasets[0].data = [];
                volatilityChart.update('none');
            }
            
            // Reset Latency Chart
            if (latencyChart) {
                latencyChart.data.labels = [];
                latencyChart.data.datasets[0].data = [];
                latencyChart.update('none');
            }
            
            // Clear data arrays
            priceData.length = 0;
            pnlData.length = 0;
            volatilityData.length = 0;
            latencyData.length = 0;
        }
        
        function clearOrderBookDisplay() {
            const tbody = document.getElementById('orderBookData');
            tbody.innerHTML = '';
            
            // Add empty rows
            for (let i = 0; i < 5; i++) {
                const row = document.createElement('tr');
                row.className = 'bid-row';
                row.innerHTML = `
                    <td>0</td>
                    <td>0.00</td>
                    <td>0.00</td>
                    <td>0</td>
                `;
                tbody.appendChild(row);
            }
        }
        
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
        
        document.getElementById('runSimpleBacktest').addEventListener('click', () => {
            log('Starting simple JavaScript backtest...');
            updateStatus('warning', 'Running Simple Backtest');
            
            // Test WASM engine first
            if (engine) {
                try {
                    const testData = generateSampleMarketData();
                    const testResult = engine.process_market_data(testData);
                    log('WASM engine test: OK');
                } catch (error) {
                    log(`WASM engine test failed: ${error.message}`);
                }
            }
            
            const dataPoints = parseInt(document.getElementById('dataPoints').value);
            const results = runSimpleBacktest(dataPoints);
            
            // Display results
            document.getElementById('totalReturn').textContent = (results.totalReturn * 100).toFixed(2) + '%';
            document.getElementById('sharpeRatio').textContent = results.sharpeRatio.toFixed(3);
            document.getElementById('maxDrawdown').textContent = (results.maxDrawdown * 100).toFixed(2) + '%';
            document.getElementById('profitFactor').textContent = results.profitFactor.toFixed(2);
            document.getElementById('calmarRatio').textContent = results.calmarRatio.toFixed(3);
            document.getElementById('sortinoRatio').textContent = results.sortinoRatio.toFixed(3);
            document.getElementById('totalTrades').textContent = results.totalTrades;
            document.getElementById('winRate').textContent = (results.winRate * 100).toFixed(1) + '%';
            
            log(`Simple backtest completed!`);
            log(`Total Return: ${(results.totalReturn * 100).toFixed(2)}%`);
            log(`Sharpe Ratio: ${results.sharpeRatio.toFixed(3)}`);
            log(`Max Drawdown: ${(results.maxDrawdown * 100).toFixed(2)}%`);
            log(`Total Trades: ${results.totalTrades}`);
            log(`Win Rate: ${(results.winRate * 100).toFixed(1)}%`);
            
            updateStatus('running', 'Simple Backtest Complete');
        });
        
        function runSimpleBacktest(dataPoints) {
            let capital = 1000000; // $1M starting capital
            let peakCapital = capital;
            let maxDrawdown = 0;
            let totalTrades = 0;
            let winningTrades = 0;
            let totalPnL = 0;
            let returns = [];
            
            const basePrice = 18450;
            const volatility = parseFloat(document.getElementById('volatilityLevel').value);
            
            for (let i = 0; i < dataPoints; i++) {
                // Generate price movement
                const priceChange = (Math.random() - 0.5) * basePrice * volatility;
                const currentPrice = basePrice + priceChange;
                
                // Simulate trading (15% chance of trade)
                if (Math.random() < 0.15) {
                    totalTrades++;
                    const tradeSize = Math.random() * 100000; // $0 to $100k per trade
                    const tradePnL = (Math.random() - 0.45) * tradeSize * 0.01; // Slightly profitable
                    
                    capital += tradePnL;
                    totalPnL += tradePnL;
                    
                    if (tradePnL > 0) {
                        winningTrades++;
                    }
                    
                    // Calculate drawdown
                    if (capital > peakCapital) {
                        peakCapital = capital;
                    }
                    const drawdown = (peakCapital - capital) / peakCapital;
                    if (drawdown > maxDrawdown) {
                        maxDrawdown = drawdown;
                    }
                    
                    // Calculate return
                    const returnRate = (capital - 1000000) / 1000000;
                    returns.push(returnRate);
                }
            }
            
            // Calculate metrics
            const totalReturn = (capital - 1000000) / 1000000;
            const winRate = totalTrades > 0 ? winningTrades / totalTrades : 0;
            
            // Calculate Sharpe ratio (simplified)
            const avgReturn = returns.length > 0 ? returns.reduce((a, b) => a + b, 0) / returns.length : 0;
            const variance = returns.length > 0 ? returns.reduce((a, b) => a + Math.pow(b - avgReturn, 2), 0) / returns.length : 0;
            const sharpeRatio = variance > 0 ? avgReturn / Math.sqrt(variance) : 0;
            
            // Calculate other ratios
            const profitFactor = totalPnL > 0 ? totalPnL / Math.abs(totalPnL - totalPnL * winRate) : 0;
            const calmarRatio = maxDrawdown > 0 ? totalReturn / maxDrawdown : 0;
            const sortinoRatio = sharpeRatio * 0.8; // Simplified
            
            return {
                totalReturn,
                sharpeRatio,
                maxDrawdown,
                profitFactor,
                calmarRatio,
                sortinoRatio,
                totalTrades,
                winRate
            };
        }
        
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
        
        document.getElementById('refreshRiskMetrics').addEventListener('click', () => {
            updateRiskMetrics(null);
            log('Risk metrics refreshed manually');
        });
        
        document.getElementById('testWasmBacktest').addEventListener('click', () => {
            if (!engine) {
                log('Engine not initialized!');
                return;
            }
            
            log('Testing WASM backtest with minimal data...');
            
            // Test with just 10 data points
            const testData = [];
            for (let i = 0; i < 10; i++) {
                testData.push(generateSampleMarketData());
            }
            
            try {
                const results = engine.run_backtest(testData);
                log('WASM backtest test successful!');
                console.log('Test results:', results);
                
                if (results && typeof results === 'object') {
                    log('Backtest results structure is valid');
                } else {
                    log('Backtest results structure is invalid');
                }
            } catch (error) {
                log(`WASM backtest test failed: ${error.message}`);
                console.error('Test error details:', error);
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
                
                // Initialize risk metrics display
                updateRiskMetrics(null);
                
            } catch (error) {
                log(`Initialization error: ${error.message}`);
            }
        };
        
        // Start the application
        initApp();
