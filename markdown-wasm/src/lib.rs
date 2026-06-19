use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Heading { level: usize, text: String },
    Paragraph(String),
    List(Vec<String>),
}

#[wasm_bindgen]
pub fn parse_markdown(_input: &str) -> String {
    // TODO: 手工实现 Markdown AST 设计、状态机解析和 HTML 渲染。
    "<p>TODO: implement markdown parser by hand</p>".to_string()
}

#[allow(dead_code)]
fn parse_document(_input: &str) -> Vec<Node> {
    // TODO: 手工实现行级解析，输出 AST 节点。
    Vec::new()
}

#[allow(dead_code)]
fn parse_heading(_line: &str) -> Option<(usize, &str)> {
    // TODO: 手工实现标题识别逻辑。
    None
}

#[allow(dead_code)]
fn parse_list_item(_line: &str) -> Option<&str> {
    // TODO: 手工实现列表项识别逻辑。
    None
}

#[allow(dead_code)]
fn flush_paragraph(_nodes: &mut Vec<Node>, _paragraph_lines: &mut Vec<String>) {
    // TODO: 手工实现段落归并逻辑。
}

#[allow(dead_code)]
fn flush_list(_nodes: &mut Vec<Node>, _list_items: &mut Vec<String>) {
    // TODO: 手工实现列表归并逻辑。
}

#[allow(dead_code)]
fn render_document(_nodes: &[Node]) -> String {
    // TODO: 手工实现 AST 到 HTML 的渲染。
    String::new()
}

#[allow(dead_code)]
fn render_node(_node: &Node) -> String {
    // TODO: 手工实现单个 AST 节点的 HTML 输出。
    String::new()
}

#[allow(dead_code)]
fn render_inline(_input: &str) -> String {
    // TODO: 手工实现行内元素与 HTML 转义。
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_markdown_returns_placeholder_html_before_manual_implementation() {
        let html = parse_markdown("# Hello Rust");
        assert!(html.contains("TODO"));
    }

    #[test]
    #[ignore = "等待手工补全 AST 设计与状态机解析逻辑"]
    fn parses_lists_and_bold_text() {
        let _html = parse_markdown("- **safe**\n- fast");
    }

    #[test]
    #[ignore = "等待手工补全 HTML 转义逻辑"]
    fn escapes_html_during_render() {
        let _html = parse_markdown("<script>");
    }
}
