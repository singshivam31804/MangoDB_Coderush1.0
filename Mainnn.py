import asyncio
from main import MarketDataFeed
from order import OrderBook
from Str import MarketMakingStrategy
from e import ExecutionEngine
from u import rolling_volatility
from m import sharpe_ratio, max_drawdown

order_book = OrderBook()
strategy = MarketMakingStrategy()
execution = ExecutionEngine()
prices_history = []

async def on_tick(tick):
    # Update order book
    order_book.update_order(tick['bid_price'], tick['bid_size'], 'bid')
    order_book.update_order(tick['ask_price'], tick['ask_size'], 'ask')

    mid = order_book.mid_price()
    prices_history.append(mid)
    vol = rolling_volatility(prices_history)

    # Generate quotes
    bid_price, bid_size, ask_price, ask_size = strategy.quote(mid, vol, execution.inventory)

    # Simulate fills
    execution.fill_order(bid_price, bid_size, 'buy')
    execution.fill_order(ask_price, ask_size, 'sell')

    print(f"Inventory: {execution.inventory}, Cash: {execution.cash}")

async def main():
    feed = MarketDataFeed("N.csv")
    await feed.stream(on_tick, delay_ms=1)

asyncio.run(main())
