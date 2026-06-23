use std::{collections::HashMap, future::Future, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tracing::{info, warn};

/// KV 参考答案说明：
///
/// 1. 这是一份完整参考实现，目的是帮助你对照理解。
/// 2. 它单独放在 `kv-store/reference.rs`，不替换你现在的练习骨架。
/// 3. 重点值得自己再写一遍的是：
///    - `Arc<RwLock<HashMap<...>>>` 的共享状态设计
///    - 命令解析器
///    - 多连接并发处理

pub type SharedStore = Arc<RwLock<HashMap<String, String>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Get(String),
    Set(String, String),
    Delete(String),
    Ping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    Unknown(String),
    MissingArgument(&'static str),
}

pub fn new_store() -> SharedStore {
    Arc::new(RwLock::new(HashMap::new()))
}

/// 把一行文本协议解析成 `Command`。
///
/// 协议非常简单：
/// - `GET key`
/// - `SET key value`
/// - `DEL key`
/// - `PING`
pub fn parse_command(input: &[u8]) -> Result<Command, ParseError> {
    let line = std::str::from_utf8(input)
        .map(str::trim)
        .map_err(|_| ParseError::Unknown("non utf-8 input".to_string()))?;

    if line.is_empty() {
        return Err(ParseError::Empty);
    }

    let mut parts = line.split_whitespace();
    let command = parts.next().unwrap().to_uppercase();

    match command.as_str() {
        "GET" => {
            let key = parts.next().ok_or(ParseError::MissingArgument("GET"))?;
            Ok(Command::Get(key.to_string()))
        }
        "SET" => {
            let key = parts.next().ok_or(ParseError::MissingArgument("SET"))?;
            // `SET` 后面剩下的内容都视为 value，允许 value 中包含空格。
            let value = parts.collect::<Vec<_>>().join(" ");
            if value.is_empty() {
                return Err(ParseError::MissingArgument("SET"));
            }
            Ok(Command::Set(key.to_string(), value))
        }
        "DEL" => {
            let key = parts.next().ok_or(ParseError::MissingArgument("DEL"))?;
            Ok(Command::Delete(key.to_string()))
        }
        "PING" => Ok(Command::Ping),
        other => Err(ParseError::Unknown(other.to_string())),
    }
}

/// 对已经解析好的命令执行读写操作，并返回文本协议响应。
pub async fn apply_command(store: &SharedStore, command: Command) -> String {
    match command {
        Command::Get(key) => {
            let store = store.read().await;
            match store.get(&key) {
                Some(value) => format!("{value}\n"),
                None => "$nil\n".to_string(),
            }
        }
        Command::Set(key, value) => {
            let mut store = store.write().await;
            store.insert(key, value);
            "+OK\n".to_string()
        }
        Command::Delete(key) => {
            let mut store = store.write().await;
            if store.remove(&key).is_some() {
                "+OK\n".to_string()
            } else {
                "$nil\n".to_string()
            }
        }
        Command::Ping => "+PONG\n".to_string(),
    }
}

/// 服务端主循环：
/// - 等待新的 TCP 连接
/// - 每接入一个连接，就启动一个异步任务处理它
/// - 可选地监听一个关闭信号，方便测试里优雅退出
pub async fn serve(
    listener: TcpListener,
    store: SharedStore,
    shutdown: impl Future<Output = ()> + Send,
) -> std::io::Result<()> {
    let local_addr = listener.local_addr()?;
    info!(%local_addr, "kv server listening");

    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                info!("shutdown signal received");
                return Ok(());
            }
            accept_result = listener.accept() => {
                let (socket, peer) = accept_result?;
                let store = store.clone();

                tokio::spawn(async move {
                    if let Err(error) = handle_connection(socket, store).await {
                        warn!(%peer, %error, "connection handler exited with error");
                    }
                });
            }
        }
    }
}

/// 单连接处理流程：
/// 1. 逐行读取命令；
/// 2. 调用解析器；
/// 3. 执行命令；
/// 4. 把响应写回客户端。
pub async fn handle_connection(socket: TcpStream, store: SharedStore) -> std::io::Result<()> {
    let peer = socket.peer_addr().ok();
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut buffer = Vec::new();

    loop {
        buffer.clear();
        let read = reader.read_until(b'\n', &mut buffer).await?;

        // read == 0 表示客户端已经关闭连接。
        if read == 0 {
            if let Some(peer) = peer {
                warn!(%peer, "client disconnected");
            }
            return Ok(());
        }

        match parse_command(&buffer) {
            Ok(command) => {
                let response = apply_command(&store, command).await;
                writer.write_all(response.as_bytes()).await?;
            }
            Err(error) => {
                if let Some(peer) = peer {
                    warn!(%peer, ?error, "invalid command");
                }
                writer
                    .write_all(format!("-ERR {:?}\n", error).as_bytes())
                    .await?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::TcpStream,
        sync::oneshot,
    };

    #[test]
    fn parse_set_command() {
        let command = parse_command(b"SET name Rust\n").unwrap();
        assert_eq!(
            command,
            Command::Set("name".to_string(), "Rust".to_string())
        );
    }

    #[tokio::test]
    async fn apply_commands_reads_and_writes_values() {
        let store = new_store();
        let response = apply_command(&store, Command::Set("name".into(), "Rust".into())).await;
        assert_eq!(response, "+OK\n");

        let response = apply_command(&store, Command::Get("name".into())).await;
        assert_eq!(response, "Rust\n");
    }

    #[tokio::test]
    async fn server_handles_multiple_clients() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let store = new_store();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let server = tokio::spawn(async move {
            serve(listener, store, async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
        });

        let writer = TcpStream::connect(address).await.unwrap();
        let mut reader = BufReader::new(writer);
        reader
            .get_mut()
            .write_all(b"SET lang Rust\n")
            .await
            .unwrap();

        let mut response = String::new();
        reader.read_line(&mut response).await.unwrap();
        assert_eq!(response, "+OK\n");

        let mut second_client = BufReader::new(TcpStream::connect(address).await.unwrap());
        second_client
            .get_mut()
            .write_all(b"GET lang\n")
            .await
            .unwrap();

        let mut get_response = String::new();
        second_client.read_line(&mut get_response).await.unwrap();
        assert_eq!(get_response, "Rust\n");

        let _ = shutdown_tx.send(());
        server.abort();
    }
}
