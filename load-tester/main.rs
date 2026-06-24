use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Error, Method};

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

    let mut progress = create_progress_bar(args.requests as u64);
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

    let total_res = execute_load_test(&args, &_client, &mut progress).await;

    // 统计结果
    let success_count = total_res.iter().filter(|r| r.is_ok()).count();
    let fail_count = total_res.len() - success_count;

    progress.finish_with_message(format!(
        "done success: {}, fail: {}",
        success_count, fail_count
    ));
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

type RequestResult = Result<u128, Error>;

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
        Ok(_resp) => {
            // 结果写日志 TODO
            return Result::Ok(elapsed.as_millis());
        }
        Err(e) => return Result::Err(e),
    }
}

async fn execute_load_test(
    args: &Args,
    client: &Client,
    process: &mut ProgressBar,
) -> Vec<RequestResult> {
    let execute_by_group = async |cnt: u64| -> Vec<RequestResult> {
        let mut que = vec![];

        for _ in 0..args.concurrency {
            let res = send_one_request(&args.url, client, &args.http_method, &args.param);
            que.push(res);
        }

        let res = join_all(que).await;

        return res;
    };

    let mut total_res = vec![];

    let groups = (args.requests / args.concurrency) as u64;
    let left = (args.requests % args.concurrency) as u64;

    for _ in 0..groups {
        let res = execute_by_group(args.concurrency as u64).await;
        // 更新进度
        process.inc(args.concurrency as u64);

        // 把结果合并到总结果中
        for ele in res {
            total_res.push(ele);
        }
    }

    let res = execute_by_group(left).await;
    // 更新进度
    process.inc(args.concurrency as u64);

    // 把结果合并到总结果中
    for ele in res {
        total_res.push(ele);
    }

    total_res
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
