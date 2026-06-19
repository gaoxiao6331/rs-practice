use clap::Parser;
use rs_practice::kv_store::new_store;

#[derive(Debug, Parser)]
#[command(name = "kv_server", about = "A minimal kv-server skeleton")]
struct Args {
    #[arg(long, default_value = "127.0.0.1:6379")]
    addr: String,
}

fn main() {
    let args = Args::parse();
    let _store = new_store();
    println!("Listening on {}...", args.addr);
    println!("TODO: 手工实现 TcpListener、协议解析和并发连接处理。");
}
