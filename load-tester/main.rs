use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

#[derive(Debug, Clone, Parser)]
#[command(name = "my_hey", about = "A minimal load-tester skeleton")]
struct Args {
    url: String,
    #[arg(short = 'c', long, default_value_t = 10)]
    concurrency: usize,
    #[arg(short = 'n', long, default_value_t = 100)]
    requests: usize,
    #[arg(long, default_value_t = 5_000)]
    timeout_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let progress = create_progress_bar(args.requests as u64);
    let _client = Client::builder()
        .timeout(Duration::from_millis(args.timeout_ms))
        .build()?;

    println!("target: {}", args.url);
    println!("concurrency: {}", args.concurrency);
    println!("requests: {}", args.requests);
    println!("timeout_ms: {}", args.timeout_ms);
    // TODO: 在这里调用“执行压测”的异步函数。
    // TODO: 这个函数内部应负责并发调度，并在过程中推进 progress。
    // TODO: 真正的 reqwest 请求不要直接写在 print_placeholder_report() 里。
    progress.finish_with_message("TODO: hand-write async request scheduling");
    print_placeholder_report(args.requests);
    Ok(())
}

// TODO: 在这里新增一个异步函数，例如 execute_load_test(...)。
// TODO: 这里是整个压测的主流程入口，负责：
// TODO: 1. 按 requests/concurrency 调度任务
// TODO: 2. 收集每次请求的结果和耗时
// TODO: 3. 汇总统计数据后返回给 main()
//
// TODO: 如需分层，再继续在下面新增一个“单次请求”函数，例如 send_one_request(...)。
// TODO: 真正调用 client.get(...).send().await 的位置应该在那个函数里。

fn create_progress_bar(total: u64) -> ProgressBar {
    let progress = ProgressBar::new(total);
    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%)",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("##-");
    progress.set_style(style);
    progress
}

fn print_placeholder_report(total_requests: usize) {
    // TODO: 这里只负责展示最终统计结果。
    // TODO: 不要在这里发送请求；请求应该在 execute_load_test/send_one_request 之类的函数里完成。
    // TODO: 后续把入参改成“统计结果对象”，而不是只有 total_requests。
    println!("\n== Load Test Report ==");
    println!("target requests : {}", total_requests);
    println!("successful      : TODO");
    println!("failed          : TODO");
    println!("elapsed         : TODO");
    println!("throughput      : TODO");
    println!("p90             : TODO");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cli_args() {
        let args = Args::parse_from(["my_hey", "-c", "8", "-n", "20", "https://tiktok.com/"]);
        assert_eq!(args.concurrency, 8);
        assert_eq!(args.requests, 20);
    }

    #[test]
    fn creates_progress_bar_with_expected_length() {
        let progress = create_progress_bar(12);
        assert_eq!(progress.length(), Some(12));
    }
}
