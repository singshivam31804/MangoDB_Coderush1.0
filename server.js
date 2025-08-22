const express = require('express');
const cors = require('cors');
const path = require('path');

const app = express();
const PORT = 5000;

// Enable CORS for all routes
app.use(cors());

// Serve static files from the current directory
app.use(express.static(__dirname));

// Parse JSON bodies
app.use(express.json());

// Chat endpoint
app.post('/chat', (req, res) => {
    const { message } = req.body;
    
    if (!message) {
        return res.status(400).json({ error: 'Message is required' });
    }

    // Simple chat logic - you can enhance this with more sophisticated responses
    const lowerMessage = message.toLowerCase();
    let reply = '';

    if (lowerMessage.includes('hello') || lowerMessage.includes('hi')) {
        reply = 'Hello! I\'m your HFT trading assistant. How can I help you today?';
    } else if (lowerMessage.includes('start') || lowerMessage.includes('run')) {
        reply = 'I can help you start the engine. Click the "Start Engine" button to begin trading operations.';
    } else if (lowerMessage.includes('backtest') || lowerMessage.includes('test')) {
        reply = 'To run a backtest, click the "Run Backtest" button. You can adjust the number of data points in the parameters section.';
    } else if (lowerMessage.includes('metrics') || lowerMessage.includes('performance')) {
        reply = 'The performance metrics are displayed in the dashboard. You can see portfolio value, PnL, risk scores, and more.';
    } else if (lowerMessage.includes('latency') || lowerMessage.includes('speed')) {
        reply = 'Latency optimization tools are available in the "Latency Optimization" section. You can benchmark performance and simulate FPGA/GPU acceleration.';
    } else if (lowerMessage.includes('risk') || lowerMessage.includes('var')) {
        reply = 'Risk metrics including VaR, Expected Shortfall, and leverage are shown in the "Risk Metrics" section.';
    } else if (lowerMessage.includes('order book') || lowerMessage.includes('l2')) {
        reply = 'The order book shows live L2 market data. It\'s displayed in the "Order Book" section with bid/ask levels.';
    } else if (lowerMessage.includes('volatility') || lowerMessage.includes('vol')) {
        reply = 'Volatility is tracked in real-time and displayed in the "Volatility & Risk" chart. The system uses EWMA and GARCH models.';
    } else if (lowerMessage.includes('help') || lowerMessage.includes('what can you do')) {
        reply = 'I can help you navigate the interface, explain features, and guide you through using the HFT trading system. Try asking about metrics, backtesting, latency, or risk management.';
    } else {
        reply = 'I understand you\'re asking about: "' + message + '". Could you please be more specific about what you\'d like to know about the HFT trading system?';
    }

    res.json({ reply });
});

// Health check endpoint
app.get('/health', (req, res) => {
    res.json({ status: 'OK', message: 'HFT Trading Server is running' });
});

// Serve the main page
app.get('/', (req, res) => {
    res.sendFile(path.join(__dirname, 'index.html'));
});

// Start the server
app.listen(PORT, () => {
    console.log(`🚀 HFT Trading Server running on http://localhost:${PORT}`);
    console.log(`📊 Chat endpoint: http://localhost:${PORT}/chat`);
    console.log(`💚 Health check: http://localhost:${PORT}/health`);
});

module.exports = app;
