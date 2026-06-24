use std::error::Error as StdError;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Error, Method, Response};

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

    let _client = Client::builder()
        .timeout(Duration::from_millis(args.timeout_ms))
        .build()?;

    println!("target: {}", args.url);
    println!("concurrency: {}", args.concurrency);
    println!("requests: {}", args.requests);
    println!("timeout_ms: {}", args.timeout_ms);

    let mut progress = create_progress_bar(args.requests as u64);

    // TODO: 在这里调用“执行压测”的异步函数。
    // TODO: 这个函数内部应负责并发调度，并在过程中推进 progress。
    // TODO: 真正的 reqwest 请求不要直接写在 print_placeholder_report() 里。

    let test_res = execute_load_test(&args, &_client, &mut progress).await;

    progress.finish_with_message("DONE!");
    print_placeholder_report(test_res);
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

type ElapsedTime = u128;

type RequestResult = Result<ElapsedTime, ()>;

async fn append_to_log(msg: String) {
    if cfg!(debug_assertions) {
        println!("{}", msg);
    }
    // release 异步写文件 TODO
}

async fn append_success_log(resp: Response) {
    let res = resp.text().await;
    let msg = match res {
        Ok(text) => text,
        Err(e) => e.to_string(),
    };
    append_to_log(format!("success: {}", msg)).await;
}

async fn append_error_log(e: Error) {
    let mut msg = format!(
        "request failed: display={}, debug={:?}, is_timeout={}, is_connect={}, url={:?}",
        e,
        e,
        e.is_timeout(),
        e.is_connect(),
        e.url()
    );

    let mut source = e.source();
    while let Some(err) = source {
        msg.push_str(&format!("\ncaused by: {}", err));
        source = err.source();
    }

    append_to_log(msg).await;
}

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
            // 结果写日志
            append_success_log(resp).await;
            Result::Ok(elapsed.as_millis())
        }
        Err(e) => {
            append_error_log(e).await;
            Result::Err(())
        }
    }
}

async fn execute_load_test(
    args: &Args,
    client: &Client,
    process: &mut ProgressBar,
) -> (Vec<RequestResult>, u128) {
    let now = Instant::now();

    let execute_by_group = async |cnt: u64| -> Vec<RequestResult> {
        let mut que = vec![];

        for _ in 0..cnt {
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

    let elapsed = now.elapsed();

    let res = execute_by_group(left).await;
    // 更新进度
    process.inc(args.concurrency as u64);

    // 把结果合并到总结果中
    for ele in res {
        total_res.push(ele);
    }

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

    // TODO: 这里只负责展示最终统计结果。
    // TODO: 不要在这里发送请求；请求应该在 execute_load_test/send_one_request 之类的函数里完成。
    // TODO: 后续把入参改成“统计结果对象”，而不是只有 total_requests。
    println!("\n== Load Test Report ==");
    println!("target requests : {}", total_res.len());
    println!("successful      : {}", success_count);
    println!("failed          : {}", fail_count);
    println!("elapsed         : {}", elapsed_avg);
    println!("throughput      : {}", throughput);
    println!("p90             : {}", p90);
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
