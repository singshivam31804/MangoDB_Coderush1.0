use std::collections::{BTreeMap, VecDeque};
use crate::{MarketData, OrderSide};

#[derive(Debug, Clone)]
pub struct Level {
    pub price: f64,
    pub quantity: f64,
    pub timestamp: f64,
}



#[derive(Debug, Clone)]
pub struct OrderBook {
    bids: BTreeMap<String, VecDeque<Level>>, // Symbol -> Bid Levels
    asks: BTreeMap<String, VecDeque<Level>>, // Symbol -> Ask Levels
    last_update_time: f64, // Renamed from last_update for clarity
    mid_price: f64,
    spread: f64,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_time: 0.0,
            mid_price: 0.0,
            spread: 0.0,
        }
    }

    // This method was added as per the requirements.
    pub fn update(&mut self, market_data: &MarketData) {
        self.update_market_data(market_data);
    }

    pub fn update_market_data(&mut self, market_data: &MarketData) {
        let symbol = &market_data.symbol;

        // Update bid side
        if market_data.bid_price > 0.0 && market_data.bid_size > 0.0 {
            let bid_level = Level {
                price: market_data.bid_price,
                quantity: market_data.bid_size,
                timestamp: market_data.timestamp,
            };

            self.bids.entry(symbol.clone())
                .or_insert_with(VecDeque::new)
                .push_back(bid_level);
        }

        // Update ask side
        if market_data.ask_price > 0.0 && market_data.ask_size > 0.0 {
            let ask_level = Level {
                price: market_data.ask_price,
                quantity: market_data.ask_size,
                timestamp: market_data.timestamp,
            };

            self.asks.entry(symbol.clone())
                .or_insert_with(VecDeque::new)
                .push_back(ask_level);
        }

        // Maintain book depth (This method needs to be defined elsewhere or implemented here)
        // For now, assuming it exists and handles pruning old data or limiting depth.
        // self.maintain_book_depth(symbol);

        // Update last update time
        self.last_update_time = market_data.timestamp;

        // Recalculate derived metrics after updates
        self.update_derived_metrics(symbol);
    }

    // Placeholder for maintain_book_depth if it's meant to be part of OrderBook
    // fn maintain_book_depth(&mut self, symbol: &str) {
    //     // Implementation to limit the number of levels or remove stale data
    // }

    fn update_derived_metrics(&mut self, symbol: &str) {
        let best_bid = self.get_best_bid(symbol).unwrap_or(0.0);
        let best_ask = self.get_best_ask(symbol).unwrap_or(0.0);

        if best_bid > 0.0 && best_ask > 0.0 {
            self.mid_price = (best_bid + best_ask) / 2.0;
            self.spread = best_ask - best_bid;
        } else if best_bid > 0.0 {
            self.mid_price = best_bid;
            self.spread = 0.0;
        } else if best_ask > 0.0 {
            self.mid_price = best_ask;
            self.spread = 0.0;
        } else {
            self.mid_price = 0.0;
            self.spread = 0.0;
        }
    }

    pub fn get_best_bid(&self, symbol: &str) -> Option<f64> {
        self.bids.get(symbol).and_then(|levels| levels.back().map(|level| level.price))
    }

    pub fn get_best_ask(&self, symbol: &str) -> Option<f64> {
        self.asks.get(symbol).and_then(|levels| levels.front().map(|level| level.price))
    }

    pub fn get_mid_price(&self) -> f64 {
        self.mid_price
    }

    pub fn get_spread(&self) -> f64 {
        self.spread
    }

    pub fn calculate_imbalance(&self, symbol: &str) -> f64 {
        let bids = self.bids.get(symbol).map_or(0.0, |levels| levels.iter().map(|level| level.quantity).sum::<f64>());
        let asks = self.asks.get(symbol).map_or(0.0, |levels| levels.iter().map(|level| level.quantity).sum::<f64>());

        if bids + asks > 0.0 {
            (bids - asks) / (bids + asks)
        } else {
            0.0
        }
    }

    pub fn calculate_depth_ratio(&self, symbol: &str) -> f64 {
        let total_bid_depth: f64 = self.bids.get(symbol).map_or(0.0, |levels| levels.iter()
            .map(|level| level.price * level.quantity)
            .sum());
        let total_ask_depth: f64 = self.asks.get(symbol).map_or(0.0, |levels| levels.iter()
            .map(|level| level.price * level.quantity)
            .sum());

        if total_ask_depth > 0.0 {
            total_bid_depth / total_ask_depth
        } else {
            1.0
        }
    }

    pub fn calculate_book_pressure(&self, symbol: &str) -> f64 {
        let bid_pressure: f64 = self.bids.get(symbol).map_or(0.0, |levels| levels.iter()
            .enumerate()
            .map(|(i, level)| level.quantity / (i + 1) as f64)
            .sum());

        let ask_pressure: f64 = self.asks.get(symbol).map_or(0.0, |levels| levels.iter()
            .enumerate()
            .map(|(i, level)| level.quantity / (i + 1) as f64)
            .sum());

        if ask_pressure > 0.0 {
            (bid_pressure - ask_pressure) / (bid_pressure + ask_pressure)
        } else {
            0.0
        }
    }

    pub fn get_volume_weighted_price(&self, symbol: &str, side: &OrderSide, volume: f64) -> f64 {
        let levels = match side {
            OrderSide::Buy => self.asks.get(symbol),
            OrderSide::Sell => self.bids.get(symbol),
        };

        if levels.is_none() {
            return 0.0;
        }

        let mut remaining_volume = volume;
        let mut total_cost = 0.0;
        let mut filled_volume = 0.0;

        let book_levels = levels.unwrap();

        for level in book_levels.iter() {
            if remaining_volume <= 0.0 {
                break;
            }

            let fill_volume = remaining_volume.min(level.quantity);
            total_cost += fill_volume * level.price;
            remaining_volume -= fill_volume;
            filled_volume += fill_volume;
        }

        if filled_volume > 0.0 {
            total_cost / filled_volume
        } else {
            0.0
        }
    }

    pub fn get_stats(&self, symbol: &str) -> crate::OrderBookStats {
        crate::OrderBookStats {
            bid_ask_spread: self.spread,
            mid_price: self.mid_price,
            imbalance: self.calculate_imbalance(symbol),
            depth_ratio: self.calculate_depth_ratio(symbol),
            book_pressure: self.calculate_book_pressure(symbol),
        }
    }

    pub fn get_default_stats(&self) -> crate::OrderBookStats {
        crate::OrderBookStats {
            bid_ask_spread: self.spread,
            mid_price: self.mid_price,
            imbalance: 0.0,
            depth_ratio: 1.0,
            book_pressure: 0.0,
        }
    }
}