use clap::Parser;

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

fn main() {
    let args = Args::parse();
    println!("target: {}", args.url);
    println!("concurrency: {}", args.concurrency);
    println!("requests: {}", args.requests);
    println!("timeout_ms: {}", args.timeout_ms);
    println!("TODO: 手工实现 tokio::spawn、mpsc 聚合、reqwest 请求和统计输出。");
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
}
