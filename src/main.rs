use clap::Parser;
use eyre::Result;
use futures_util::StreamExt;
use nix::{getsockopt_impl, libc, setsockopt_impl, sockopt_impl, sys::socket::setsockopt};
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
use std::time::UNIX_EPOCH;
use std::{
    collections::BTreeSet,
    sync::mpsc::{self, Receiver, Sender},
};
use tungstenite::connect;
use url::Url;

const FLUSH_SIZE: usize = 2;

#[derive(clap::Parser)]
struct Cli {
    #[clap(short, long)]
    process: Process,
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Process {
    AsyncWs,
    PollWsMultithread,
    BusyPollWs,
}

#[tokio::main]
async fn main() -> Result<()> {
    default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

    let (tx, rx) = mpsc::channel();

    match cli.process {
        Process::AsyncWs => {
            println!("开始进行异步 WebSocket 延迟测试...");
            tokio::spawn(async_ws(tx));
            record_res("async_ws.csv", rx);
        }
        Process::PollWsMultithread => {
            println!("开始进行多线程 WebSocket 延迟测试...");
            let num_threads = num_cpus::get();
            poll_ws_multithread(num_threads, tx);
            record_res_deduplication("poll_ws_multithread.csv", rx);
        }
        Process::BusyPollWs => {
            println!("开始进行忙轮询 WebSocket 延迟测试...");
            std::thread::spawn(|| busy_poll_ws(tx));
            record_res("busy_poll_ws.csv", rx);
        }
    }

    Ok(())
}

async fn async_ws(tx: Sender<u128>) {
    let mut client = OkxClientV5::builder().build().unwrap();

    let params: OkxWsRequest<OkxBookData> =
        OkxWsRequest::<OkxBookData>::builder("subscribe", vec![OkxArg::new("bbo-tbt", "BTC-USDT")])
            .build();

    let mut stream = <OkxClientV5 as DataSubscriber<OkxWsResponse<OkxBookData>>>::subscribe_data(
        &mut client,
        params,
    )
    .await
    .unwrap();

    loop {
        while let Some(Ok(data_vec)) = stream.next().await {
            if let Some(data) = data_vec.first() {
                tx.send(calc_elapsed(data.timestamp)).ok();
            }
        }
    }
}

/// 使用多个同步线程进行 WebSocket 延迟测试
///
/// 每个线程独立连接到 WebSocket 并轮询消息。
fn poll_ws_multithread(num_threads: usize, tx: Sender<u128>) {
    for _ in 0..num_threads {
        let tx = tx.clone();

        std::thread::spawn(move || {
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

            // 忽略第一个成功消息
            socket.read().ok();

            loop {
                if let Ok(msg) = socket.read()
                    && let Ok(text) = msg.to_text()
                    && let Ok(resp) = simd_json::from_slice::<OkxWsDataResponse<OkxBookData>>(
                        &mut text.to_string().into_bytes(),
                    )
                    && let Ok(book_data_vec) = resp
                        .data
                        .into_iter()
                        .map(BookData::try_from)
                        .collect::<Result<Vec<_>>>()
                    && let Some(data) = book_data_vec.first()
                {
                    tx.send(calc_elapsed(data.timestamp)).ok();
                }
            }
        });
    }
}

fn busy_poll_ws(tx: Sender<u128>) {
    let url_str = "wss://wspap.okx.com/ws/v5/public";
    let url = Url::parse(url_str).expect("Invalid URL");
    let (mut socket, _) = connect(url).expect("Failed to connect");

    let stream_ref = match socket.get_mut() {
        tungstenite::stream::MaybeTlsStream::Rustls(tls_stream) => tls_stream.get_mut(),
        _ => {
            panic!("Expected a TLS stream, ensure you have a TLS feature enabled for tungstenite.")
        }
    };

    sockopt_impl!(
        BusyPoll,
        Both,
        libc::SOL_SOCKET,
        libc::SO_BUSY_POLL,
        libc::c_int
    );
    // 设置 SO_BUSY_POLL。需要root
    setsockopt(&stream_ref, BusyPoll, &10_000_000).unwrap();

    let params: OkxWsRequest<OkxBookData> =
        OkxWsRequest::<OkxBookData>::builder("subscribe", vec![OkxArg::new("bbo-tbt", "BTC-USDT")])
            .build();
    socket
        .send(tungstenite::Message::Text(
            simd_json::serde::to_string(&params).unwrap().into(),
        ))
        .unwrap();

    // 忽略第一个成功消息
    socket.read().ok();

    loop {
        if let Ok(msg) = socket.read()
            && let Ok(text) = msg.to_text()
            && let Ok(resp) = simd_json::from_slice::<OkxWsDataResponse<OkxBookData>>(
                &mut text.to_string().into_bytes(),
            )
            && let Ok(book_data_vec) = resp
                .data
                .into_iter()
                .map(BookData::try_from)
                .collect::<Result<Vec<_>>>()
            && let Some(data) = book_data_vec.first()
        {
            tx.send(calc_elapsed(data.timestamp)).ok();
        }
    }
}

/// 将延迟数据记录到 CSV 文件
fn record_res(path: &str, rx: Receiver<u128>) {
    let mut writer = csv::Writer::from_path(path).unwrap();

    // 写入表头
    writer.write_record(["duration_ms"]).unwrap();

    let mut i = 0_usize;
    loop {
        if let Ok(elapsed) = rx.recv() {
            writer.write_record(&[elapsed.to_string()]).ok();

            i += 1;
            if i.is_multiple_of(FLUSH_SIZE) {
                writer.flush().ok();
                println!("已记录 {i} 条数据");
            }
        }
    }
}

fn record_res_deduplication(path: &str, rx: Receiver<u128>) {
    let mut writer = csv::Writer::from_path(path).unwrap();

    // 写入表头
    writer.write_record(["duration_ms"]).unwrap();

    let mut btree = BTreeSet::new();

    let mut i = 0_usize;
    loop {
        if let Ok(elapsed) = rx.recv()
            && btree.insert(elapsed)
        {
            writer.write_record(&[elapsed.to_string()]).ok();

            i += 1;
            if i.is_multiple_of(FLUSH_SIZE) {
                writer.flush().ok();
                println!("已记录 {i} 条数据");
            }
        }
    }
}

fn calc_elapsed(ts: u128) -> u128 {
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    now.abs_diff(ts)
}
