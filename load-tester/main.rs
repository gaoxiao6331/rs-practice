use std::sync::atomic::AtomicU32;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Method};
use std::sync::Arc;
use tokio::task::JoinSet;

use tracing::{error, info};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::fmt;

#[derive(Debug, Clone, Parser)]
#[command(name = "my_hey", about = "A minimal load-tester skeleton")]
struct Args {
    url: String,
    #[arg(short = 'c', long, default_value_t = 10)]
    concurrency: usize,
    #[arg(short = 'n', long, default_value_t = 100)]
    requests: usize,
    #[arg(short = 't', long, default_value_t = 5_000)]
    timeout_ms: u64,
    #[arg(short = 'm', long, default_value_t = Method::GET)]
    http_method: Method,
    #[arg(short = 'p', long, default_value_t = String::new())]
    param: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    assert!(
        args.concurrency <= args.requests,
        "concurrency > requests -> {} > {}",
        args.concurrency,
        args.requests
    );

    let file_appender = rolling::never("logs", "load-test.log");

    let (non_blocking_writer, _guard) = non_blocking(file_appender);

    fmt()
        .with_writer(non_blocking_writer)
        .with_target(false)
        .init();

    info!("Starting the test application...");

    let _client = Client::builder()
        .timeout(Duration::from_millis(args.timeout_ms))
        .build()?;

    println!("target: {}", args.url);
    println!("concurrency: {}", args.concurrency);
    println!("requests: {}", args.requests);
    println!("timeout_ms: {}", args.timeout_ms);

    let progress = create_progress_bar(args.requests as u64);

    let test_res = execute_load_test(&args, &_client, progress).await;

    print_placeholder_report(test_res);
    Ok(())
}

type ElapsedTime = u128;

type RequestResult = Result<ElapsedTime, ()>;

async fn send_one_request(
    url: &str,
    client: &Client,
    method: &Method,
    param: &str,
) -> RequestResult {
    // 开始计时
    let start = Instant::now();
    // 使用http client 发送请求
    let res = client
        .request(method.to_owned(), url)
        .header("Content-Type", "application/json")
        .body(param.to_owned())
        .send()
        .await;
    // 计算请求事件
    let elapsed = start.elapsed();

    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                info!("size: {}", resp.content_length().unwrap_or(0));
            }
            Result::Ok(elapsed.as_millis())
        }
        Err(e) => {
            error!(%e);
            Result::Err(())
        }
    }
}

async fn execute_load_test(
    args: &Args,
    http_client: &Client,
    progress_bar: ProgressBar,
) -> (Vec<RequestResult>, u128) {
    // 结果数组
    let mut total_res = vec![];

    // 并发数
    let concurrency = args.concurrency;

    // 总请求数
    let total_req_count = args.requests as u32;

    // 完成的请求数
    let req_count_finished = Arc::new(AtomicU32::new(0));

    let mut set = JoinSet::new();

    let url = Arc::new(args.url.clone());

    let method = Arc::new(args.http_method.clone());

    let param = Arc::new(args.param.clone());

    let client = http_client.clone();

    let now = Instant::now();

    for _ in 0..concurrency {
        let u = Arc::clone(&url);

        let m = Arc::clone(&method);

        let p = Arc::clone(&param);

        let f = Arc::clone(&req_count_finished);

        let c = client.clone();

        let b = progress_bar.clone();

        set.spawn(async move {
            let mut req_resps = vec![];

            while f.fetch_add(1, std::sync::atomic::Ordering::SeqCst) < total_req_count {
                let resp = send_one_request(&*u, &c, &*m, &*p).await;
                req_resps.push(resp);
                b.inc(1);
            }
            req_resps
        });
    }

    while let Some(r) = set.join_next().await {
        match r {
            Ok(v) => {
                total_res.extend(v);
            }
            Err(e) => {
                error!(%e, "fail to join")
            }
        }
    }

    let elapsed = now.elapsed();

    progress_bar.finish_with_message("DONE!");

    (total_res, elapsed.as_millis())
}

fn create_progress_bar(total: u64) -> ProgressBar {
    let progress = ProgressBar::new(total);
    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%)",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("##-");
    progress.set_style(style);
    progress.set_position(0);
    progress.tick();
    progress
}

fn print_placeholder_report(test_res: (Vec<RequestResult>, u128)) {
    let (total_res, elapsed) = test_res;

    // 统计结果
    let mut success = total_res
        .iter()
        .filter_map(|r| match r {
            Ok(r) => Some(*r),
            Err(_) => None,
        })
        .collect::<Vec<ElapsedTime>>();

    success.sort();

    let success_count = (&success).len();

    let elapsed_avg = if success_count > 0 {
        success.iter().sum::<ElapsedTime>() as f64 / success_count as f64
    } else {
        f64::INFINITY
    };

    let fail_count = total_res.len() - success_count;

    let p90 = if success.is_empty() {
        u128::MIN
    } else {
        let p90_idx = (success.len() * 9).div_ceil(10) - 1;
        success[p90_idx]
    };

    let throughput = success_count as f64 / elapsed as f64;

    println!("\n== Load Test Report ==");
    println!("target requests : {}", total_res.len());
    println!("successful      : {}", success_count);
    println!("failed          : {}", fail_count);
    println!("elapsed         : {}", elapsed_avg);
    println!("throughput      : {}", throughput);
    println!("p90             : {}", p90);
}
