use std::{collections::HashMap, future::Future, sync::Arc};

use thiserror::Error;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tracing::info;

pub type SharedStore = Arc<RwLock<HashMap<String, String>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Get(String),
    Set(String, String),
    Delete(String),
    Ping,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("empty command")]
    Empty,
    #[error("unknown command: {0}")]
    Unknown(String),
    #[error("missing argument for {0}")]
    MissingArgument(&'static str),
}

pub fn new_store() -> SharedStore {
    // TODO: 手工确认共享状态的所有权边界与锁粒度是否满足你的设计。
    Arc::new(RwLock::new(HashMap::new()))
}

pub fn parse_command(_input: &[u8]) -> Result<Command, ParseError> {
    // TODO: 手工实现自定义协议解析，把字节流解析成 `Command`。
    Err(ParseError::Unknown(
        "TODO: implement command parser by hand".to_string(),
    ))
}

pub async fn apply_command(_store: &SharedStore, _command: Command) -> String {
    // TODO: 手工实现 `Arc<RwLock<HashMap<...>>>` 的读写逻辑与响应格式。
    "-ERR TODO: implement command execution by hand\n".to_string()
}

pub async fn serve(
    listener: TcpListener,
    store: SharedStore,
    shutdown: impl Future<Output = ()> + Send,
) -> std::io::Result<()> {
    let local_addr = listener.local_addr()?;
    info!(%local_addr, "kv server listening");

    // 这里只保留服务生命周期与监听骨架，连接处理逻辑留给手写部分完成。
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                info!("shutdown signal received");
                return Ok(());
            }
            accept_result = listener.accept() => {
                let (_socket, _peer) = accept_result?;
                let _store = store.clone();
                // TODO: 手工实现每个 TCP 连接的任务派发与错误日志。
            }
        }
    }
}

pub async fn handle_connection(_socket: TcpStream, _store: SharedStore) -> std::io::Result<()> {
    // TODO: 手工实现裸 TCP 字节流读取、命令解析和响应写回。
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_is_created_empty() {
        let _store = new_store();
    }

    #[tokio::test]
    #[ignore = "等待手工补全共享状态读写逻辑"]
    async fn apply_commands_reads_and_writes_values() {
        let store = new_store();
        let _response = apply_command(&store, Command::Set("name".into(), "Rust".into())).await;
    }

    #[tokio::test]
    #[ignore = "等待手工补全 TCP 协议解析与多客户端处理逻辑"]
    async fn server_handles_multiple_clients() {
        let _listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    }
}
