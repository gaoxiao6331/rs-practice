use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use reqwest::Client;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "my_hey",
    about = "A small async HTTP load tester written in Rust"
)]
struct Args {
    /// The target URL to benchmark.
    url: String,
    /// Maximum number of in-flight requests.
    #[arg(short = 'c', long, default_value_t = 10)]
    concurrency: usize,
    /// Total number of requests to execute.
    #[arg(short = 'n', long, default_value_t = 100)]
    requests: usize,
    /// Timeout for each request in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    timeout_ms: u64,
}

#[derive(Debug, Clone, Default)]
struct RequestResult {
    success: bool,
    latency_ms: f64,
    status_code: Option<u16>,
    error: Option<String>,
}

#[derive(Debug, Default)]
struct Summary {
    success: usize,
    failure: usize,
    latencies_ms: Vec<f64>,
    sample_errors: Vec<String>,
    status_codes: Vec<u16>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::builder()
        .timeout(Duration::from_millis(args.timeout_ms))
        .build()?;

    let (summary, elapsed) = run_load_test(&args, &client, false).await?;
    print_report(&summary, elapsed, args.requests);
    Ok(())
}

async fn run_load_test(
    args: &Args,
    _client: &Client,
    hidden_progress: bool,
) -> Result<(Summary, Duration)> {
    // TODO: 手工实现重点手写区
    // 1. 用 `tokio::spawn` 调度并发请求
    // 2. 用 `mpsc channel` 聚合每个请求的结果
    // 3. 用 `?` 优雅传播网络错误
    // 这里先保留进度条初始化，便于后续直接接入真实执行逻辑。
    let progress = create_progress_bar(args.requests as u64, hidden_progress);
    progress.finish_with_message("TODO: implement async load test core");

    let summary = Summary::default();
    Ok((summary, Duration::default()))
}

#[allow(dead_code)]
async fn perform_request(_client: Client, _url: String, _timeout: Duration) -> RequestResult {
    // TODO: 手工实现单次 HTTP 请求执行与耗时采样。
    RequestResult::default()
}

fn create_progress_bar(total: u64, hidden: bool) -> ProgressBar {
    let progress = if hidden {
        ProgressBar::with_draw_target(Some(total), ProgressDrawTarget::hidden())
    } else {
        ProgressBar::new(total)
    };

    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%)",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("##-");

    progress.set_style(style);
    progress
}

fn print_report(summary: &Summary, elapsed: Duration, total_requests: usize) {
    // 这部分保留输出结构，方便手写核心逻辑后直接填充统计结果。
    println!("\n== Load Test Report ==");
    println!("target requests : {}", total_requests);
    println!("successful      : {}", summary.success);
    println!("failed          : {}", summary.failure);
    println!("elapsed         : {:.2?}", elapsed);
    println!("throughput      : TODO");
    println!("fastest         : TODO");
    println!("slowest         : TODO");
    println!("average         : TODO");
    println!("p90             : TODO");
    println!("p99             : TODO");

    if !summary.status_codes.is_empty() {
        println!(
            "status sample   : {:?}",
            &summary.status_codes[..summary.status_codes.len().min(10)]
        );
    }

    if !summary.sample_errors.is_empty() {
        println!("errors          : {:?}", summary.sample_errors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_is_created_for_requested_total() {
        let progress = create_progress_bar(20, true);
        assert_eq!(progress.length(), Some(20));
    }

    #[tokio::test]
    #[ignore = "等待手工补全并发调度、通道聚合与延迟统计逻辑"]
    async fn load_test_hits_local_server() {
        let args = Args {
            url: "http://127.0.0.1:3000/health".to_string(),
            concurrency: 4,
            requests: 20,
            timeout_ms: 1_000,
        };
        let client = Client::builder().build().unwrap();

        let (_summary, _elapsed) = run_load_test(&args, &client, true).await.unwrap();
    }
}
