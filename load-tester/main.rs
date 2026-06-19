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
    progress.finish_with_message("TODO: hand-write async request scheduling");
    print_placeholder_report(args.requests);
    Ok(())
}

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
    // TODO: 手工实现 tokio::spawn、mpsc 聚合、reqwest 请求和统计输出。
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
        let args = Args::parse_from(["my_hey", "-c", "8", "-n", "20", "http://localhost"]);
        assert_eq!(args.concurrency, 8);
        assert_eq!(args.requests, 20);
    }

    #[test]
    fn creates_progress_bar_with_expected_length() {
        let progress = create_progress_bar(12);
        assert_eq!(progress.length(), Some(12));
    }
}
