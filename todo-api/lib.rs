use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTodo {
    pub title: String,
    pub completed: bool,
}

pub fn project_note() -> &'static str {
    // TODO: 手工实现 axum 路由、AppError、SQLite 初始化和 CRUD。
    "TODO: implement todo api by hand"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_note_mentions_todo() {
        assert!(project_note().contains("TODO"));
    }
}
