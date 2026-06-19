use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub type SharedStore = Arc<RwLock<HashMap<String, String>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Get(String),
    Set(String, String),
}

pub fn new_store() -> SharedStore {
    // TODO: 手工实现时再确认锁粒度和共享状态设计。
    Arc::new(RwLock::new(HashMap::new()))
}

pub fn parse_command(_input: &[u8]) -> Result<Command, &'static str> {
    Err("TODO: 手工实现自定义协议解析")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_is_created_empty() {
        let store = new_store();
        assert!(store.read().unwrap().is_empty());
    }
}
