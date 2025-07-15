use eyre::Result;
use futures_util::StreamExt;
use squant::client::{
    DataSubscriber,
    okx::{
        OkxClientV5,
        model::{OkxArg, OkxWebSocketSubscribeRequest},
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = OkxClientV5::builder().build()?;

    let params = OkxWebSocketSubscribeRequest::builder(
        "subscribe",
        vec![OkxArg::new("candle1D", "BTC-USDT")],
    )
    .build();
    let mut stream = client.subscribe_data(params).await?;

    while let Some(data) = stream.next().await {
        println!("debug0: {data:?}");
    }

    Ok(())
}
