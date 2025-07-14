use super::*;
use crate::{
    Exchange, Symbol,
    client::{ExchangeTrait, okx::OkxKlineDataApiParamsV5},
    data::{CandleData, RawData, MarketDataTrait},
};
use anyhow::{Result, bail};
use ringbuf::{HeapRb, LocalRb, storage::Heap};
use std::collections::VecDeque;

pub struct SmaCrossoverOptions<E: ExchangeTrait<CandleData>> {
    pub exchange: E,
    pub symbol: Symbol,
    pub short_window_size: usize, // 短期SMA窗口
    pub long_window_size: usize,  // 长期SMA窗口
}

/// 简单移动平均线 (SMA) 交叉策略
struct SmaCrossover<E: ExchangeTrait<CandleData>> {
    options: SmaCrossoverOptions<E>,
    short_sma: f64,
    long_sma: f64,
    position: i64, // 持仓数量
}

impl<E: ExchangeTrait<CandleData>> StrategyInternalTrait<CandleData> for SmaCrossover<E> {
    async fn init(&mut self) -> Result<()> {
        // 获取至少 long_window_size 个数据点
        let mut data = Vec::with_capacity(100);
        let params = OkxKlineDataApiParamsV5::builder()
            .inst_id(&self.options.symbol)
            .bar("1m") // 1分钟K线
            .limit(100) // 多取一些数据以便计算
            .build();

        let options = &self.options;
        options.exchange.get_datas(&options.symbol, . ).await?;

        todo!()
    }

    async fn handle_market_data(&mut self, data: CandleData) -> Result<()> {
        todo!()
    }

    async fn generate_order(&self) -> Option<Order> {
        todo!()
    }
}

impl<E: ExchangeTrait<CandleData>> SmaCrossover<E> {}
