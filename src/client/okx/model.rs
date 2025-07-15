#![allow(dead_code)]

use super::*;
use crate::{Timestamp, order::Side};
use bon::Builder;
use bytestring::ByteString;
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub(super) const OKX_CODE_SUCCESS: &str = "0";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OkxHttpResponse<D: RawData> {
    pub code: ByteString,
    pub msg: ByteString,
    pub data: Vec<D>,
}

impl<R: RawData> DataResponse for OkxHttpResponse<R> {
    type Request = R;
}

impl<D: RawData> RawData for OkxHttpResponse<D> {
    type Data = Vec<D::Data>;
}

#[derive(Builder)]
pub struct OkxHttpCandleDataRequest {
    /// 产品ID，如 BTC-USDT
    #[builder(start_fn, into)]
    pub inst_id: ByteString,
    /// 时间粒度，默认值1m
    /// 如 [1m/3m/5m/15m/30m/1H/2H/4H]
    /// 香港时间开盘价k线：[6H/12H/1D/2D/3D/1W/1M/3M]
    /// UTC时间开盘价k线：[/6Hutc/12Hutc/1Dutc/2Dutc/3Dutc/1Wutc/1Mutc/3Mutc]
    pub bar: Option<ByteString>,
    /// 请求此时间戳之前（更旧的数据）的分页内容，传的值为对应接口的ts
    pub after: Option<Timestamp>,
    /// 请求此时间戳之后（更新的数据）的分页内容，传的值为对应接口的ts, 单独使用时，会返回最新的数据。
    pub before: Option<Timestamp>,
    /// 分页返回的结果集数量，最大为300，不填默认返回100条
    pub limit: Option<usize>,
}

impl RawData for OkxHttpCandleDataRequest {
    type Data = super::CandleData;
}

/// 订阅的频道
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxArg {
    /// 频道名
    /// candle3M
    /// candle1M
    /// candle1W
    /// candle1D
    /// candle2D
    /// candle3D
    /// candle5D
    /// candle12H
    /// candle6H
    /// candle4H
    /// candle2H
    /// candle1H
    /// candle30m
    /// candle15m
    /// candle5m
    /// candle3m
    /// candle1m
    /// candle1s
    /// candle3Mutc
    /// candle1Mutc
    /// candle1Wutc
    /// candle1Dutc
    /// candle2Dutc
    /// candle3Dutc
    /// candle5Dutc
    /// candle12Hutc
    /// candle6Hutc
    pub channel: ByteString,
    /// 产品ID，例如 "BTC-USDT"。
    pub inst_id: ByteString,
}

impl OkxArg {
    pub fn new(channel: impl Into<ByteString>, inst_id: impl Into<ByteString>) -> Self {
        Self {
            channel: channel.into(),
            inst_id: inst_id.into(),
        }
    }
}

#[derive(Default, Builder, Serialize)]
pub struct OkxWsRequest<D> {
    /// 操作
    /// subscribe
    /// unsubscribe
    #[builder(start_fn, into)]
    pub op: ByteString,
    /// 请求订阅的频道列表
    #[builder(start_fn, into)]
    pub args: Vec<OkxArg>,

    /// 消息的唯一标识。
    /// 用户提供，返回参数中会返回以便于找到相应的请求。
    /// 字母（区分大小写）与数字的组合，可以是纯字母、纯数字且长度必须要在1-32位之间。
    pub id: Option<ByteString>,

    #[serde(skip)]
    #[builder(skip)]
    _phantom: PhantomData<D>,
}

impl<D: RawData> RawData for OkxWsRequest<D> {
    type Data = D::Data;
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxWsResponse<D> {
    /// 消息的唯一标识。
    pub id: Option<ByteString>,

    /// 事件类型
    pub event: ByteString,

    /// 错误码
    pub code: Option<ByteString>,

    /// 错误消息
    pub msg: Option<ByteString>,

    /// 订阅的频道
    pub arg: Option<OkxArg>,

    /// WebSocket连接ID
    pub conn_id: ByteString,

    #[serde(skip)]
    _phantom: PhantomData<D>,
}

impl<D: RawData> RawData for OkxWsResponse<D> {
    type Data = Result<Vec<D::Data>>;
}

impl<D> DataResponse for OkxWsResponse<D> {
    type Request = OkxWsRequest<D>;
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxWsDataResponse<D> {
    pub arg: OkxArg,
    pub action: Option<ByteString>,
    pub data: Vec<D>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxTradeData {
    pub inst_id: ByteString,
    pub trade_id: ByteString,
    pub px: ByteString,
    pub sz: ByteString,
    pub side: ByteString,
    pub ts: ByteString,
}

impl TryFrom<OkxTradeData> for crate::data::TradeData {
    type Error = eyre::Report;

    fn try_from(value: OkxTradeData) -> Result<Self> {
        let timestamp = value
            .ts
            .parse()
            .wrap_err("Failed to parse trade timestamp")?;
        let price = value
            .px
            .parse::<f64>()
            .wrap_err_with(|| format!("Failed to parse trade price: '{}'", value.px))?;
        let quantity = value
            .sz
            .parse::<f64>()
            .wrap_err_with(|| format!("Failed to parse trade volume: '{}'", value.sz))?;
        let side = Side::try_from(value.side.as_ref())?;

        Ok(Self {
            trade_id: value.trade_id,
            symbol: value.inst_id,
            price,
            quantity,
            side,
            timestamp,
        })
    }
}

/// 0. 价格,
/// 1. 数量,
/// 2. 流动性订单数量,
/// 3. 订单数量
pub type Level = (ByteString, ByteString, ByteString, ByteString);

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxBookData {
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
    pub ts: ByteString,

    /// 检验和
    pub checksum: Option<i128>,

    /// 上一个推送的序列号。仅适用 books，books-l2-tbt，books50-l2-tbt
    pub prev_seq_id: Option<i128>,

    /// 推送的序列号
    pub seq_id: Option<i128>,
}

impl RawData for OkxBookData {
    type Data = crate::data::BookData;
}

impl TryFrom<OkxBookData> for crate::data::BookData {
    type Error = eyre::Report;

    fn try_from(value: OkxBookData) -> Result<Self> {
        let parse_levels = |levels: &Vec<Level>| -> Result<Vec<(f64, f64)>> {
            levels
                .iter()
                .map(|(price_str, size_str, _, _)| {
                    let price = price_str
                        .parse::<f64>()
                        .wrap_err("Failed to parse book price")?;
                    let size = size_str
                        .parse::<f64>()
                        .wrap_err("Failed to parse book size")?;
                    Ok((price, size))
                })
                .collect()
        };

        let timestamp = value
            .ts
            .parse()
            .wrap_err("Failed to parse book timestamp")?;
        let bids = parse_levels(&value.bids).wrap_err("Failed to parse bids")?;
        let asks = parse_levels(&value.asks).wrap_err("Failed to parse asks")?;

        Ok(Self {
            timestamp,
            bids,
            asks,
        })
    }
}

/// 0.开始时间，Unix时间戳的毫秒数
/// 1.开盘价
/// 2.最高价
/// 3.最低价
/// 4.收盘价
/// 5.交易量（以币为单位）
/// 6.交易量（以计价货币为单位）
/// 7.交易量（以计价货币为单位，适用于合约）
/// 8.K线状态 (1: a confirmed candle)
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OkxCandleData(
    ByteString,
    ByteString,
    ByteString,
    ByteString,
    ByteString,
    ByteString,
    ByteString,
    ByteString,
    ByteString,
);

impl RawData for OkxCandleData {
    type Data = super::CandleData;
}

impl TryFrom<OkxCandleData> for super::CandleData {
    type Error = eyre::Report;

    fn try_from(value: OkxCandleData) -> Result<Self> {
        let timestamp = value
            .0
            .parse()
            .wrap_err("Failed to parse candle timestamp")?;
        let open = value
            .1
            .parse::<f64>()
            .wrap_err("Failed to parse open price")?;
        let high = value
            .2
            .parse::<f64>()
            .wrap_err("Failed to parse high price")?;
        let low = value
            .3
            .parse::<f64>()
            .wrap_err("Failed to parse low price")?;
        let close = value
            .4
            .parse::<f64>()
            .wrap_err("Failed to parse close price")?;
        let volume = value.5.parse::<f64>().wrap_err("Failed to parse volume")?;

        Ok(Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        })
    }
}
