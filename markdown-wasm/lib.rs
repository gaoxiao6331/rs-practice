use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_markdown(_input: &str) -> String {
    // TODO: 手工实现 Markdown AST、状态机解析和 HTML 渲染。
    "<p>TODO: implement markdown parser by hand</p>".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_placeholder_html() {
        let html = parse_markdown("# Hello Rust");
        assert!(html.contains("TODO"));
    }
}
