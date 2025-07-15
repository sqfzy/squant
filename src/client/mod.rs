use eyre::Result;
use futures_util::StreamExt;
use std::future::Future;

pub mod okx;

// 通常你需要为每个Request和未标准化的交易所数据实现该trait
pub trait RawData {
    type Data;
}

// 通常你需要为每个Response实现该trait
pub trait DataResponse {
    /// 每个客户端的请求参数可能不同，因此需要为每个请求结果定义它的请求参数。
    type Request;
}

pub trait DataGetter<Resp: DataResponse + RawData> {
    fn get_data(&mut self, params: Resp::Request) -> impl Future<Output = Result<Resp::Data>> + Send;
}

pub trait DataSubscriber<Resp: DataResponse + RawData> {
    fn subscribe_data(
        &mut self,
        params: Resp::Request,
    ) -> impl Future<Output = Result<impl StreamExt<Item = Resp::Data> + Send>> + Send;
}
