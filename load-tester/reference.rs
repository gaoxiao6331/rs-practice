use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use reqwest::Client;
use tokio::sync::{mpsc, Semaphore};

/// 这是压测工具的“参考答案”版本。
///
/// 说明：
/// 1. 这个文件放在 `load-tester/` 目录下，方便你和当前练习骨架并排查看。
/// 2. 它不会替换当前练习入口文件，目的是给你一份完整、可读、带详细注释的实现。
/// 3. 这里会把 requirement 里提到的 CLI 参数、进度条、报告输出都补齐。
/// 4. 真正最值得你自己重新敲一遍的是并发调度、结果聚合和错误处理这几块。

#[derive(Debug, Clone, Parser)]
#[command(name = "my_hey_reference", about = "Reference solution for the CLI load tester")]
struct Args {
    /// 目标地址，例如 http://127.0.0.1:3000/health
    url: String,
    /// 并发数，也就是同时允许多少个请求在飞行中
    #[arg(short = 'c', long, default_value_t = 10)]
    concurrency: usize,
    /// 总请求数
    #[arg(short = 'n', long, default_value_t = 100)]
    requests: usize,
    /// 每个请求的超时时间，单位毫秒
    #[arg(long, default_value_t = 5_000)]
    timeout_ms: u64,
}

/// 单个请求完成后发给聚合器的结果。
#[derive(Debug, Clone)]
struct RequestResult {
    success: bool,
    latency_ms: f64,
    status_code: Option<u16>,
    error: Option<String>,
}

/// 压测结束后的汇总结果。
#[derive(Debug, Default)]
struct Summary {
    success: usize,
    failure: usize,
    latencies_ms: Vec<f64>,
    sample_errors: Vec<String>,
    status_codes: Vec<u16>,
}

impl Summary {
    /// 聚合器每收到一个结果，就把统计信息累加到 `Summary` 里。
    fn record(&mut self, result: RequestResult) {
        if result.success {
            self.success += 1;
        } else {
            self.failure += 1;
        }

        self.latencies_ms.push(result.latency_ms);

        if let Some(code) = result.status_code {
            self.status_codes.push(code);
        }

        // 报告里只保留少量错误样本，避免输出刷屏。
        if let Some(error) = result.error {
            if self.sample_errors.len() < 5 {
                self.sample_errors.push(error);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // `reqwest` 负责 HTTP 协议封装，这部分正是 requirement 里允许库代劳的内容。
    let client = Client::builder()
        .timeout(Duration::from_millis(args.timeout_ms))
        .build()?;

    let (summary, elapsed) = run_load_test(&args, &client, false).await?;
    print_report(&summary, elapsed, args.requests);
    Ok(())
}

/// 真正执行压测的函数。
///
/// 这里做了三件核心事情：
/// 1. 用信号量限制最大并发数；
/// 2. 用 `tokio::spawn` 派发任务；
/// 3. 用 `mpsc` 把每个请求的结果发回聚合器。
async fn run_load_test(
    args: &Args,
    client: &Client,
    hidden_progress: bool,
) -> Result<(Summary, Duration)> {
    let progress = create_progress_bar(args.requests as u64, hidden_progress);
    let semaphore = Arc::new(Semaphore::new(args.concurrency.max(1)));
    let (tx, mut rx) = mpsc::channel::<RequestResult>(args.requests.max(1));
    let started_at = Instant::now();
    let mut handles = Vec::with_capacity(args.requests);

    for _ in 0..args.requests {
        // 先拿到一个 permit，再启动任务。
        // 这样可以确保同时运行的任务数不会超过 `concurrency`。
        let permit = semaphore.clone().acquire_owned().await?;
        let tx = tx.clone();
        let client = client.clone();
        let url = args.url.clone();
        let progress = progress.clone();
        let timeout = Duration::from_millis(args.timeout_ms);

        handles.push(tokio::spawn(async move {
            let result = perform_request(client, url, timeout).await;
            let _ = tx.send(result).await;
            progress.inc(1);

            // permit 在任务结束时释放，这样下一个等待中的请求就可以开始。
            drop(permit);
        }));
    }

    // 原始发送端可以关闭了，只保留任务里克隆出来的发送端。
    drop(tx);

    // 单独起一个聚合任务，持续从 channel 里收集结果。
    let aggregator = tokio::spawn(async move {
        let mut summary = Summary::default();
        while let Some(result) = rx.recv().await {
            summary.record(result);
        }
        summary
    });

    for handle in handles {
        handle.await?;
    }

    progress.finish_with_message("load test complete");
    let summary = aggregator.await?;
    Ok((summary, started_at.elapsed()))
}

/// 执行一次 HTTP GET 请求，并记录耗时和状态。
async fn perform_request(client: Client, url: String, timeout: Duration) -> RequestResult {
    let started_at = Instant::now();

    // 这里故意又包了一层 `tokio::time::timeout`，
    // 是为了演示如何把“超时”作为一类独立错误处理。
    let result = tokio::time::timeout(timeout, client.get(url).send()).await;
    let latency_ms = started_at.elapsed().as_secs_f64() * 1000.0;

    match result {
        Ok(Ok(response)) => {
            let status = response.status();
            RequestResult {
                success: status.is_success(),
                latency_ms,
                status_code: Some(status.as_u16()),
                error: (!status.is_success()).then(|| format!("non-success status: {}", status)),
            }
        }
        Ok(Err(error)) => RequestResult {
            success: false,
            latency_ms,
            status_code: None,
            error: Some(error.to_string()),
        },
        Err(_) => RequestResult {
            success: false,
            latency_ms,
            status_code: None,
            error: Some(format!("request timed out after {} ms", timeout.as_millis())),
        },
    }
}

/// 构造一个终端进度条。
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

/// 打印一份格式化报告。
fn print_report(summary: &Summary, elapsed: Duration, total_requests: usize) {
    let mut latencies = summary.latencies_ms.clone();
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let fastest = latencies.first().copied().unwrap_or_default();
    let slowest = latencies.last().copied().unwrap_or_default();
    let average = if latencies.is_empty() {
        0.0
    } else {
        latencies.iter().sum::<f64>() / latencies.len() as f64
    };
    let p90 = percentile(&latencies, 90.0);
    let p99 = percentile(&latencies, 99.0);
    let qps = if elapsed.is_zero() {
        0.0
    } else {
        total_requests as f64 / elapsed.as_secs_f64()
    };

    println!("\n== Load Test Report ==");
    println!("target requests : {}", total_requests);
    println!("successful      : {}", summary.success);
    println!("failed          : {}", summary.failure);
    println!("elapsed         : {:.2?}", elapsed);
    println!("throughput      : {:.2} req/s", qps);
    println!("fastest         : {:.2} ms", fastest);
    println!("slowest         : {:.2} ms", slowest);
    println!("average         : {:.2} ms", average);
    println!("p90             : {:.2} ms", p90);
    println!("p99             : {:.2} ms", p99);

    if !summary.sample_errors.is_empty() {
        println!("errors          : {:?}", summary.sample_errors);
    }
}

/// 从已排序延迟数组里取出简单百分位值。
fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let rank = ((percentile / 100.0) * sorted_values.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted_values.len() - 1);
    sorted_values[index]
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};

    #[test]
    fn percentile_uses_expected_rank() {
        let sorted = [10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(percentile(&sorted, 50.0), 30.0);
        assert_eq!(percentile(&sorted, 90.0), 50.0);
    }

    #[tokio::test]
    async fn load_test_hits_local_server() {
        let app = Router::new().route("/", get(|| async { "ok" }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let args = Args {
            url: format!("http://{address}/"),
            concurrency: 4,
            requests: 20,
            timeout_ms: 1_000,
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .unwrap();

        let (summary, _) = run_load_test(&args, &client, true).await.unwrap();
        server.abort();

        assert_eq!(summary.success, 20);
        assert_eq!(summary.failure, 0);
        assert_eq!(summary.latencies_ms.len(), 20);
    }
}
