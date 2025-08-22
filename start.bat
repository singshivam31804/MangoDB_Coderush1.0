@echo off
echo 🚀 Starting HFT Market Making Bot...
echo.

echo 📦 Installing Node.js dependencies...
npm install

echo.
echo 🔧 Building WebAssembly module...
wasm-pack build --target web --out-dir pkg

echo.
echo 🌐 Starting server on http://localhost:5000
echo.
echo 💡 Open your browser and navigate to: http://localhost:5000
echo.
echo Press Ctrl+C to stop the server
echo.

node server.js
