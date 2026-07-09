use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use bytes::Bytes;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Method};
use tokio::task::JoinSet;
use tracing::{error, info, warn};
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

    let client = Client::builder()
        .timeout(Duration::from_millis(args.timeout_ms))
        .build()?;

    println!("target: {}", args.url);
    println!("concurrency: {}", args.concurrency);
    println!("requests: {}", args.requests);
    println!("timeout_ms: {}", args.timeout_ms);

    let progress = create_progress_bar(args.requests as u64);
    let test_res = execute_load_test(&args, &client, progress).await;

    print_placeholder_report(test_res);
    Ok(())
}

type ElapsedTime = u128;
type RequestResult = Result<ElapsedTime, ()>;

async fn send_one_request(
    url: &str,
    client: &Client,
    method: Method,
    param: Bytes, // Passed by value, cheap to clone
) -> RequestResult {
    let start = Instant::now();
    let res = if method == Method::GET {
        client.request(method, url).send().await
    } else {
        client
            .request(method, url)
            .header("Content-Type", "application/json")
            .body(param) // reqwest consumes Bytes without allocation
            .send()
            .await
    };

    let elapsed = start.elapsed();

    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                Result::Ok(elapsed.as_millis())
            } else {
                error!(
                    "status code: {}, error: {}",
                    resp.status(),
                    resp.text()
                        .await
                        .unwrap_or("parse response text error".to_string())
                );
                Result::Err(())
            }
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
    if args.http_method == Method::GET && !args.param.is_empty() {
        warn!("you are using GET method, param will be ignored");
    }

    let mut total_res = vec![];
    let concurrency = args.concurrency;
    let total_req_count = args.requests as u32;

    let req_count_finished = Arc::new(AtomicU32::new(0));
    let mut set = JoinSet::new();

    let url = Arc::new(args.url.clone());
    let method = args.http_method.clone();
    let param = Bytes::from(args.param.clone()); // Convert String to Bytes once
    let client = http_client.clone();
    let now = Instant::now();

    for _ in 0..concurrency {
        let u = Arc::clone(&url);
        let m = method.clone();
        let p = param.clone(); // Cloning Bytes is just a fast reference count increment
        let f = Arc::clone(&req_count_finished);
        let c = client.clone();
        let b = progress_bar.clone();

        set.spawn(async move {
            // Pre-allocate the vector with expected capacity
            let expected_capacity = (total_req_count as usize / concurrency) + 1;
            // add 50% buffer to prevent reallocation due to uneven load distribution
            let actual_capacity = (expected_capacity as f64 * 1.5) as usize;
            let mut req_resps = Vec::with_capacity(actual_capacity);

            while let Ok(_) = f.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |cur| {
                if cur < total_req_count {
                    Some(cur + 1)
                } else {
                    None
                }
            }) {
                let resp = send_one_request(&u, &c, m.clone(), p.clone()).await;
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

    let mut success = total_res
        .iter()
        .filter_map(|r| match r {
            Ok(r) => Some(*r),
            Err(_) => None,
        })
        .collect::<Vec<ElapsedTime>>();

    success.sort();

    let success_count = success.len();

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

    // Calculate throughput correctly as Requests Per Second (RPS)
    let elapsed_sec = elapsed as f64 / 1000.0;
    let throughput = if elapsed_sec > 0.0 {
        success_count as f64 / elapsed_sec
    } else {
        0.0
    };

    println!("\n== Load Test Report ==");
    println!("target requests : {}", total_res.len());
    println!("successful      : {}", success_count);
    println!("failed          : {}", fail_count);
    println!("elapsed avg (ms): {}", elapsed_avg);
    println!("throughput (RPS): {:.2}", throughput);
    println!("p90 (ms)        : {}", p90);
}
