use std::{fmt::format, sync::LazyLock};

use crate::{
    Symbol,
    client::{DataGetter, DataSubscriber},
    data::{CandleData, DataResponse, RawData, TradeData, okx::*},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD_ENGINE};
use bon::Builder;
use bytestring::ByteString;
use chrono::{DateTime, SecondsFormat, Utc};
use const_format::{concatcp, formatcp};
use eyre::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use itertools::Itertools;
use reqwest::{
    Client,
    header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
};
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use sha2::Sha256;
use tokio_websockets::Message;
use url::Url;

use super::ExchangeTrait;

pub struct OkxClientV5 {
    client: Client,
    api_key: Option<ByteString>,        // 即 OK_ACCESS_KEY
    api_passphrase: Option<ByteString>, // 即 OK_ACCESS_PASSPHRASE
    buffer: itoa::Buffer,               // 高效的整数转为字符串的缓冲区
}

impl OkxClientV5 {
    pub fn new() -> Self {
        let ok_access_key = std::env::var("OK_ACCESS_KEY").ok();

        let ok_access_passphrase = std::env::var("OK_ACCESS_PASSPHRASE").ok();

        OkxClientV5 {
            client: Client::new(),
            api_key: ok_access_key.map(Into::into),
            api_passphrase: ok_access_passphrase.map(Into::into),
            buffer: itoa::Buffer::new(),
        }
    }
}

impl ExchangeTrait for OkxClientV5 {
    const NAME: &'static str = "okx";
    const BASE_URL: &'static str = "http://www.okx.com/";
}

impl DataGetter<OkxHttpResponse<OkxHttpCandleDataRequest>> for OkxClientV5 {
    async fn get_data(
        &mut self,
        params: <OkxHttpResponse<OkxHttpCandleDataRequest> as DataResponse>::Request,
    ) -> Result<<OkxHttpResponse<OkxHttpCandleDataRequest> as RawData>::Data> {
        let OkxHttpCandleDataRequest {
            inst_id,
            bar,
            after,
            before,
            limit,
        } = params;

        let mut uri = Url::parse(concatcp!(OkxClientV5::BASE_URL, "api/v5/market/candles"))?;

        {
            let mut query = uri.query_pairs_mut();

            let buffer = &mut self.buffer;
            query.append_pair("instId", &inst_id);
            if let Some(bar) = bar {
                query.append_pair("bar", bar.as_ref());
            }
            if let Some(after) = after {
                query.append_pair("after", buffer.format(after));
            }
            if let Some(before) = before {
                query.append_pair("before", buffer.format(before));
            }
            if let Some(limit) = limit {
                query.append_pair("limit", buffer.format(limit));
            }
        }

        let resp = self
            .client
            .get(uri)
            .send()
            .await?
            .error_for_status()?
            .json::<OkxHttpResponse<OkxCandleData>>()
            .await?;

        resp.data.into_iter().map(CandleData::try_from).collect()
    }
}

impl DataSubscriber<OkxWebSocketResponse<OkxCandleData>> for OkxClientV5 {
    async fn subscribe_data(
        &mut self,
        params: <OkxWebSocketResponse<OkxCandleData> as DataResponse>::Request,
    ) -> Result<impl StreamExt<Item = <OkxWebSocketResponse<OkxCandleData> as RawData>::Data> + Send>
    {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_okx_client_v5() {
        OkxClientV5::new();
    }
}

//
//     // async fn get_datas(
//     //     &mut self,
//     //     symbol: &Symbol,
//     //     period: usize,
//     //     quantity: usize,
//     // ) -> anyhow::Result<Vec<CandleData>> {
//     //     #[derive(Debug, Deserialize)]
//     //     struct ResponseBody {
//     //         code: String,
//     //         msg: String,
//     //         data: Vec<RawKlineData>,
//     //     }
//     //
//     //     #[derive(Debug, Deserialize)]
//     //     struct RawKlineData(
//     //         String, // 0: timestamp
//     //         String, // 1: open
//     //         String, // 2: high
//     //         String, // 3: low
//     //         String, // 4: close
//     //         String, // 5: volume_base
//     //         String, // 6: volume_quote
//     //         String, // 7: volume_quote_specific
//     //         String, // 8: confirmed
//     //     );
//     //
//     //     impl TryFrom<RawKlineData> for CandleData {
//     //         type Error = anyhow::Error;
//     //
//     //         fn try_from(value: RawKlineData) -> Result<Self, Self::Error> {
//     //             Ok(CandleData {
//     //                 timestamp: value.0.parse()?,
//     //                 open: value.1.parse()?,
//     //                 high: value.2.parse()?,
//     //                 low: value.3.parse()?,
//     //                 close: value.4.parse()?,
//     //                 volume: value.5.parse()?,
//     //             })
//     //         }
//     //     }
//     //
//     //     // 构建请求
//     //     let url = format!(
//     //         "{}/market/candles?instId={}&bar={}&limit={}",
//     //         Self::BASE_URL,
//     //         symbol,
//     //         period,
//     //         quantity
//     //     );
//     //
//     //     // 发送请求
//     //     let resp = self.client.get(&url).send().await?.error_for_status()?;
//     //
//     //     // 处理响应
//     //     let body = resp.json::<ResponseBody>().await?;
//     //
//     //     Ok(body
//     //         .data
//     //         .into_iter()
//     //         .map(|raw| raw.try_into())
//     //         .try_collect()?)
//     // }
// }

// fn generate_okx_headers(
//     okx: &OkxClientV5,
//     method: &str,
//     request_path: &str,
//     body_str: &str,
// ) -> Result<HeaderMap, Box<dyn std::error::Error>> {
//     let mut headers = HeaderMap::new();
//
//     headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
//     headers.insert(
//         HeaderName::from_static("ok-access-key"),
//         HeaderValue::from_str(&okx.api_key)?,
//     );
//     headers.insert(
//         HeaderName::from_static("ok-access-passphrase"),
//         HeaderValue::from_str(&okx.api_passphrase)?,
//     );
//
//     // 1. 创建时间戳 (timestamp)
//     // 格式要求：ISO8601 格式，带毫秒，例如 2020-12-08T09:08:57.715Z
//     let now: DateTime<Utc> = Utc::now();
//     let timestamp = now.to_rfc3339_opts(SecondsFormat::Millis, true);
//
//     headers.insert(
//         HeaderName::from_static("ok-access-timestamp"),
//         HeaderValue::from_str(&timestamp)?,
//     );
//
//     // 2. 创建签名
//     let secret_key = std::env::var("OK_ACCESS_SECRET")
//         .context("OK_ACCESS_SECRET environment variable not set")?;
//     let signature = generate_okx_signature(method, request_path, body_str, &secret_key)?;
//
//     headers.insert(
//         HeaderName::from_static("ok-access-sign"),
//         HeaderValue::from_str(&signature)?,
//     );
//
//     Ok(headers)
// }
//
// /// 根据 OKX API 规则生成签名
// ///
// /// # Arguments
// /// * `method` - HTTP 请求方法，大写形式，例如 "GET", "POST"。
// /// * `request_path` - 请求的路径，包含查询参数（如果存在）。
// ///   例如："/api/v5/account/balance?ccy=BTC" 或 "/api/v5/trade/order"。
// /// * `body_str` - 请求体字符串。对于 GET 请求或没有请求体的 POST 请求，应为空字符串 ""。
// ///   对于有请求体的 POST 请求，这是 JSON 请求体的字符串形式。
// /// * `secret_key` - 你的 API SecretKey。
// ///
// /// # Returns
// /// * `Ok(OkxAuthHeaders)` 包含生成的时间戳和签名。
// /// * `Err` 如果在过程中发生错误。
// fn generate_okx_signature(
//     method: &str,
//     request_path: &str,
//     body_str: &str,
//     secret_key: &str,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     // 1. 创建时间戳 (timestamp)
//     // 格式要求：ISO8601 格式，带毫秒，例如 2020-12-08T09:08:57.715Z
//     let now: DateTime<Utc> = Utc::now();
//     let timestamp = now.to_rfc3339_opts(SecondsFormat::Millis, true);
//
//     // 2. 创建预签名字符串 (pre-hash string)
//     // 格式: timestamp + method + requestPath + body
//     // 注意：method 需要是大写
//     let prehash_string = format!(
//         "{}{}{}{}",
//         timestamp,
//         method.to_uppercase(),
//         request_path,
//         body_str
//     );
//
//     // 3. 使用 HMAC SHA256 和 SecretKey 对预签名字符串进行签名
//     type HmacSha256 = Hmac<Sha256>;
//     let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
//         .context("Failed to create HMAC SHA256 instance")?;
//     mac.update(prehash_string.as_bytes());
//     let signature_bytes = mac.finalize().into_bytes();
//
//     // 4. 将签名进行 Base64 编码
//     Ok(BASE64_STANDARD_ENGINE.encode(signature_bytes))
// }
