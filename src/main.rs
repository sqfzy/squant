use std::{fmt, str::FromStr, time::SystemTime};

use eyre::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{
    Deserialize, Deserializer,
    de::{self, SeqAccess, Visitor},
};
use squant::{
    client::{DataGetter, okx::OkxClientV5},
    data::{CandleInterval, okx::OkxHttpCandleDataRequest},
};
use tokio_websockets::Message;

#[tokio::main]
async fn main() -> Result<()> {
    let (mut client, _) = tokio_websockets::client::Builder::new()
        .uri("wss://wspap.okx.com:8443/ws/v5/business")?
        .connect()
        .await?;

    client
        .send(Message::text(
            r#"{"op":"subscribe","args":[{"channel":"candle1D","instId":"BTC-USDT"}]}"#.to_string(),
        ))
        .await?;

    while let Some(Ok(msg)) = client.next().await {
        println!("debug0: msg: {:?}", msg.as_text().unwrap());
        println!(
            "debug0: msg: {:?}",
            serde_json::from_str::<CandleResponse>(msg.as_text().unwrap())
        );
    }
    let mut okx_client = OkxClientV5::new();

    let resp = okx_client
        .get_data(OkxHttpCandleDataRequest::builder("BTC-USDT").build())
        .await?;

    println!("debug0: {resp:?}");

    Ok(())
}
