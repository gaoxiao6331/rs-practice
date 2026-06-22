use clap::Parser;

#[path = "reference.rs"]
mod reference;

/// Markdown WASM 参考答案的可运行入口。
///
/// 这里选择做成一个原生 CLI 演示程序：
/// - 方便直接 `cargo run`
/// - 能立刻验证参考解析器的输出
/// - 不影响你后续再用 `wasm-pack` 去生成真正的浏览器版本
#[derive(Debug, Parser)]
#[command(
    name = "markdown_reference",
    about = "Runnable reference solution for the markdown parser"
)]
struct Args {
    /// 待解析的 Markdown 文本。
    #[arg(long, default_value = "# Hello Rust")]
    input: String,
}

fn main() {
    let args = Args::parse();
    let html = reference::parse_markdown(&args.input);
    println!("{html}");
}
