use crate::data::{DataResponse, RawData};
use eyre::Result;
use futures_util::StreamExt;
use std::future::Future;

pub mod okx;

pub trait DataGetter<D: DataResponse + RawData> {
    fn get_data(&mut self, params: D::Request) -> impl Future<Output = Result<D::Data>> + Send;
}

pub trait DataSubscriber<D: DataResponse + RawData> {
    fn subscribe_data(
        &mut self,
        params: D::Request,
    ) -> impl Future<Output = Result<impl StreamExt<Item = D::Data> + Send>> + Send;
}
