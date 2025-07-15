use eyre::Result;
use futures_util::StreamExt;
use squant::{
    client::{
        DataSubscriber,
        okx::{
            OkxClientV5,
            model::{OkxArg, OkxBookData, OkxWsDataResponse, OkxWsRequest, OkxWsResponse},
        },
    },
    data::BookData,
};
use std::{net::TcpStream, time::UNIX_EPOCH};
use tungstenite::{connect, stream::NoDelay};
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    std::fs::create_dir_all("temp");

    let times = 100;
    let res = async_ws(times).await?;
    record_res("temp/async_ws.csv", res)?;

    let res = poll_ws(times);
    record_res("temp/poll_ws.csv", res)?;

    Ok(())
}

fn record_res(path: &str, data: Vec<u128>) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;

    // 写入表头
    writer.write_record(["duration_ms"])?;

    // 逐行写入数据
    for duration in data {
        writer.write_record(&[duration.to_string()])?;
    }

    // 确保所有内容都写入文件
    writer.flush()?;

    println!("数据已成功保存到 {path}");
    Ok(())
}

async fn async_ws(mut times: usize) -> Result<Vec<u128>> {
    let mut result = Vec::with_capacity(times);

    let mut client = OkxClientV5::builder().build()?;

    let params: OkxWsRequest<OkxBookData> =
        OkxWsRequest::<OkxBookData>::builder("subscribe", vec![OkxArg::new("bbo-tbt", "BTC-USDT")])
            .build();
    let mut stream = <OkxClientV5 as DataSubscriber<OkxWsResponse<OkxBookData>>>::subscribe_data(
        &mut client,
        params,
    )
    .await?;

    while let Some(Ok(data)) = stream.next().await {
        if times == 0 {
            break;
        }

        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let ts = data[0].timestamp;

        // println!("debug0: now: {now}, ts: {ts}");
        let elapsed = now.abs_diff(ts);
        result.push(elapsed);
        // println!("debug0: elapsed: {elapsed}");

        times -= 1;
    }

    Ok(result)
}

fn poll_ws(mut times: usize) -> Vec<u128> {
    let mut result = Vec::with_capacity(times);

    // 1. CORRECT: 定义并解析完整的、正确的 WSS URL
    let url_str = "wss://wspap.okx.com:443/ws/v5/public";
    let url = Url::parse(url_str).expect("Invalid URL");

    // 2. CORRECT: 使用 `tungstenite::connect` 来处理整个连接过程（TCP + TLS + WebSocket握手）
    //    这个函数会返回一个已经可以通信的 WebSocket<Stream>
    let (mut socket, response) = connect(url).expect("Failed to connect");

    println!("Successfully connected to OKX WebSocket API.");
    println!("HTTP Response: {}", response.status());

    // 3. KEY STEP: *在连接成功后*，获取底层流的引用并设置为非阻塞
    let stream_ref = match socket.get_mut() {
        tungstenite::stream::MaybeTlsStream::NativeTls(tls_stream) => tls_stream.get_mut(),
        _ => {
            panic!("Expected a TLS stream, ensure you have a TLS feature enabled for tungstenite.")
        }
    };
    stream_ref
        .set_nonblocking(true)
        .expect("Failed to set non-blocking on the underlying stream");

    let params: OkxWsRequest<OkxBookData> =
        OkxWsRequest::<OkxBookData>::builder("subscribe", vec![OkxArg::new("bbo-tbt", "BTC-USDT")])
            .build();
    socket
        .send(tungstenite::Message::Text(
            simd_json::serde::to_string(&params).unwrap().into(),
        ))
        .unwrap();

    // 3. 进入无限循环，不停地尝试读取消息
    let mut success = false;
    loop {
        match socket.read() {
            // A. 成功读取到消息
            Ok(msg) => {
                if !success {
                    success = true;
                    continue;
                }

                if times == 0 {
                    break;
                }

                let text = msg.to_text().unwrap();
                let resp: OkxWsDataResponse<OkxBookData> =
                    simd_json::from_slice(&mut text.to_string().into_bytes()).unwrap();
                let data = resp
                    .data
                    .into_iter()
                    .map(BookData::try_from)
                    .collect::<Result<Vec<_>>>()
                    .unwrap();
                let ts = data[0].timestamp;

                let now = std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();

                let elapsed = now.abs_diff(ts);
                result.push(elapsed);

                times -= 1;
            }
            _ => {
                continue;
            }
        }
    }

    result
}
