use eyre::Result;
use futures_util::StreamExt;
use rustls::crypto::aws_lc_rs::default_provider;
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
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::UNIX_EPOCH,
};
use tungstenite::connect;
use url::Url;

/// 程序主入口，用于启动延迟测试
#[tokio::main]
async fn main() -> Result<()> {
    default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // 确保用于存放结果的目录存在
    std::fs::create_dir_all("temp").unwrap();

    let times = 1000; // 每批次收集的数据点数量
    let num_threads = num_cpus::get(); // 获取 CPU 核心数用于多线程测试

    // println!("开始进行异步 WebSocket 延迟测试...");
    //     async_ws(times).await.ok();

    // 多线程测试默认被注释掉，可以取消注释来运行
    println!("开始进行多线程同步 WebSocket 延迟测试...");
    poll_ws_multithread(num_threads, times);

    Ok(())
}

/// 将延迟数据记录到 CSV 文件
///
/// # Arguments
/// * `path` - 输出的 CSV 文件路径
/// * `data` - 包含延迟时间（毫秒）的向量
fn record_res(path: &str, data: &Vec<u128>) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;

    // 写入表头
    writer.write_record(["duration_ms"])?;

    // 逐行写入数据
    for &duration in data {
        writer.write_record(&[duration.to_string()])?;
    }

    // 确保所有内容都写入文件
    writer.flush()?;

    println!("数据已成功保存到 {}", path);
    Ok(())
}

async fn async_ws(times: usize) -> Result<()> {
    let mut client = OkxClientV5::builder().build()?;

    let params: OkxWsRequest<OkxBookData> =
        OkxWsRequest::<OkxBookData>::builder("subscribe", vec![OkxArg::new("bbo-tbt", "BTC-USDT")])
            .build();
    let mut stream = <OkxClientV5 as DataSubscriber<OkxWsResponse<OkxBookData>>>::subscribe_data(
        &mut client,
        params,
    )
    .await?;

    let mut result = Vec::with_capacity(times);
    let mut batch_index = 0; // 使用简单的批次索引

    // 在同一个 stream 上持续循环
    while let Some(Ok(data_vec)) = stream.next().await {
        if let Some(data) = data_vec.get(0) {
            let now = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let ts = data.timestamp;

            let elapsed = now.abs_diff(ts);
            result.push(elapsed);
        }

        // 当收集到足够的数据时，记录并重置
        if result.len() >= times {
            let path = format!("temp/async_ws_{}.csv", batch_index);
            record_res(&path, &result).ok();

            result.clear();
            batch_index += 1; // 为下一个文件准备索引
        }
    }

    Ok(()) // 正常情况下，这个循环是无限的，除非 stream 中断
}

/// 使用多个同步线程进行 WebSocket 延迟测试
///
/// 每个线程独立连接到 WebSocket 并轮询消息。
fn poll_ws_multithread(num_threads: usize, times: usize) {
    let k = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(num_threads);

    for _ in 0..num_threads {
        let k = k.clone();

        let handle = std::thread::spawn(move || {
            let url_str = "wss://wspap.okx.com/ws/v5/public";
            let url = Url::parse(url_str).expect("Invalid URL");
            let (mut socket, _) = connect(url).expect("Failed to connect");

            let stream_ref = match socket.get_mut() {
                tungstenite::stream::MaybeTlsStream::Rustls(tls_stream) => tls_stream.get_mut(),
                _ => {
                    panic!(
                        "Expected a TLS stream, ensure you have a TLS feature enabled for tungstenite."
                    )
                }
            };
            stream_ref
                .set_nonblocking(true)
                .expect("Failed to set non-blocking on the underlying stream");

            let params: OkxWsRequest<OkxBookData> = OkxWsRequest::<OkxBookData>::builder(
                "subscribe",
                vec![OkxArg::new("bbo-tbt", "BTC-USDT")],
            )
            .build();
            socket
                .send(tungstenite::Message::Text(
                    simd_json::serde::to_string(&params).unwrap().into(),
                ))
                .unwrap();

            // 等待订阅成功的回应
            if let Ok(_) = socket.read() {
                // 忽略第一个成功消息
            }

            loop {
                let mut result = Vec::with_capacity(times);

                socket.read().ok();

                while result.len() < times {
                    match socket.read() {
                        Ok(msg) => {
                            let text = match msg.to_text() {
                                Ok(t) => t,
                                Err(_) => continue, // 忽略非文本消息
                            };

                            if let Ok(resp) = simd_json::from_slice::<OkxWsDataResponse<OkxBookData>>(
                                &mut text.to_string().into_bytes(),
                            ) {
                                if let Ok(book_data_vec) = resp
                                    .data
                                    .into_iter()
                                    .map(BookData::try_from)
                                    .collect::<Result<Vec<_>>>()
                                {
                                    if let Some(data) = book_data_vec.get(0) {
                                        let now = std::time::SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap()
                                            .as_millis();
                                        let ts = data.timestamp;
                                        let elapsed = now.abs_diff(ts);
                                        result.push(elapsed);
                                    }
                                }
                            }
                        }
                        Err(_) => continue,
                    }
                }

                let file_index = k.fetch_add(1, Ordering::Relaxed);
                let path = format!("temp/poll_ws_multithread_{}.csv", file_index);
                record_res(&path, &result).ok();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
