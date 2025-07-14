use crate::{
    Symbol,
    data::{CandleData, DataResponse, RawData, TradeData},
};
use eyre::Result;
use futures_util::StreamExt;
use std::future::Future;

pub mod okx;

pub trait ExchangeTrait {
    const NAME: &'static str;
    const BASE_URL: &'static str;
}

pub trait DataGetter<D: DataResponse + RawData>: ExchangeTrait {
    fn get_data(&mut self, params: D::Request) -> impl Future<Output = Result<D::Data>> + Send;
}

pub trait DataSubscriber<D: DataResponse + RawData>: ExchangeTrait {
    fn subscribe_data(
        &mut self,
        params: D::Request,
    ) -> impl Future<Output = Result<impl StreamExt<Item = D::Data> + Send>> + Send;
}
