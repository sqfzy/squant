use crate::{data::MarketDataTrait, order::Order};

pub mod sma;

// pub trait StrategyTrait<D: MarketDataTrait>: StrategyInternalTrait<D> {
//     fn run(&mut self) -> anyhow::Result<()> {
//         self.init()?;
//
//         Ok(())
//     }
// }

pub(super) trait StrategyInternalTrait<D: MarketDataTrait> {
    /// Initialize the strategy with the given configuration.
    async fn init(&mut self) -> anyhow::Result<()>;

    /// Handle incoming market data.
    async fn handle_market_data(&mut self, data: D) -> anyhow::Result<()>;

    async fn generate_order(&self) -> Option<Order>;
}
