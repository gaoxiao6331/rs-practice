use wasm_bindgen::prelude::*;

/// 这份参考答案展示了一个“最小但完整”的 Markdown 解析器。
///
/// 它不是通用 Markdown 引擎，而是一个为了练习 Rust 语法、字符串处理、
/// 迭代器和 AST 设计而写的教学版本。
///
/// 支持的语法：
/// - 标题：`# title`
/// - 段落
/// - 列表：`- item`
/// - 行内粗体：`**bold**`
#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Heading { level: usize, text: String },
    Paragraph(String),
    List(Vec<String>),
}

/// 对外暴露给 JavaScript 的函数。
///
/// `wasm-bindgen` 会把它包装成 JS 可调用接口，
/// `wasm-pack build` 时会自动生成胶水代码和 `.wasm` 文件。
#[wasm_bindgen]
pub fn parse_markdown(input: &str) -> String {
    let nodes = parse_document(input);
    render_document(&nodes)
}

/// 把整篇 Markdown 文本解析成一个节点列表。
///
/// 这里采用“按行扫描 + 少量状态缓存”的策略：
/// - `paragraph_lines` 用来暂存连续段落行
/// - `list_items` 用来暂存连续列表项
/// - 碰到空行或块级语法切换时，就把缓存刷进 AST
fn parse_document(input: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut paragraph_lines = Vec::new();
    let mut list_items = Vec::new();

    for raw_line in input.lines() {
        let line = raw_line.trim();

        if line.is_empty() {
            flush_paragraph(&mut nodes, &mut paragraph_lines);
            flush_list(&mut nodes, &mut list_items);
            continue;
        }

        if let Some((level, text)) = parse_heading(line) {
            flush_paragraph(&mut nodes, &mut paragraph_lines);
            flush_list(&mut nodes, &mut list_items);
            nodes.push(Node::Heading {
                level,
                text: text.to_string(),
            });
            continue;
        }

        if let Some(item) = parse_list_item(line) {
            flush_paragraph(&mut nodes, &mut paragraph_lines);
            list_items.push(item.to_string());
            continue;
        }

        // 如果既不是标题也不是列表，就先把它当段落内容缓存起来。
        flush_list(&mut nodes, &mut list_items);
        paragraph_lines.push(line.to_string());
    }

    // 文件结束后，别忘了把最后一批缓存刷进 AST。
    flush_paragraph(&mut nodes, &mut paragraph_lines);
    flush_list(&mut nodes, &mut list_items);
    nodes
}

/// 尝试把一行识别为标题。
fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.chars().take_while(|ch| *ch == '#').count();

    if hashes == 0 || hashes > 6 {
        return None;
    }

    let remainder = line.get(hashes..)?.trim_start();
    if remainder.is_empty() {
        None
    } else {
        Some((hashes, remainder))
    }
}

/// 尝试把一行识别为列表项。
fn parse_list_item(line: &str) -> Option<&str> {
    line.strip_prefix("- ").or_else(|| line.strip_prefix("* "))
}

/// 如果段落缓存不为空，就把它合并成一个 `Paragraph` 节点。
fn flush_paragraph(nodes: &mut Vec<Node>, paragraph_lines: &mut Vec<String>) {
    if paragraph_lines.is_empty() {
        return;
    }

    let paragraph = paragraph_lines.join(" ");
    paragraph_lines.clear();
    nodes.push(Node::Paragraph(paragraph));
}

/// 如果列表缓存不为空，就把它合并成一个 `List` 节点。
fn flush_list(nodes: &mut Vec<Node>, list_items: &mut Vec<String>) {
    if list_items.is_empty() {
        return;
    }

    let items = std::mem::take(list_items);
    nodes.push(Node::List(items));
}

/// 渲染整棵 AST。
fn render_document(nodes: &[Node]) -> String {
    nodes.iter().map(render_node).collect::<Vec<_>>().join("\n")
}

/// 渲染单个节点。
fn render_node(node: &Node) -> String {
    match node {
        Node::Heading { level, text } => format!("<h{level}>{}</h{level}>", render_inline(text)),
        Node::Paragraph(text) => format!("<p>{}</p>", render_inline(text)),
        Node::List(items) => {
            let rendered_items = items
                .iter()
                .map(|item| format!("<li>{}</li>", render_inline(item)))
                .collect::<Vec<_>>()
                .join("");
            format!("<ul>{rendered_items}</ul>")
        }
    }
}

/// 处理行内语法和 HTML 转义。
///
/// 这里演示了一个简单状态机：
/// - 默认状态：普通文本
/// - 遇到 `**`：切换粗体开关
///
/// 这不是完整 Markdown 规范实现，但很适合学习字符串遍历思路。
fn render_inline(input: &str) -> String {
    let mut html = String::new();
    let mut chars = input.chars().peekable();
    let mut strong_open = false;

    while let Some(ch) = chars.next() {
        if ch == '*' && chars.peek() == Some(&'*') {
            chars.next();

            if strong_open {
                html.push_str("</strong>");
            } else {
                html.push_str("<strong>");
            }

            strong_open = !strong_open;
            continue;
        }

        // 输出到 HTML 前，先做最基本的转义，防止原始标签直接插入页面。
        match ch {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            '"' => html.push_str("&quot;"),
            '\'' => html.push_str("&#39;"),
            _ => html.push(ch),
        }
    }

    // 如果输入里出现了不成对的 `**`，这里简单补一个闭合标签。
    if strong_open {
        html.push_str("</strong>");
    }

    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_heading_and_paragraph() {
        let html = parse_markdown("# Hello Rust\n\nRust is fast.");
        assert_eq!(html, "<h1>Hello Rust</h1>\n<p>Rust is fast.</p>");
    }

    #[test]
    fn parses_list_and_bold_text() {
        let html = parse_markdown("- **safe**\n- fast");
        assert_eq!(html, "<ul><li><strong>safe</strong></li><li>fast</li></ul>");
    }

    #[test]
    fn escapes_html() {
        let html = parse_markdown("<script>");
        assert_eq!(html, "<p>&lt;script&gt;</p>");
    }
}
