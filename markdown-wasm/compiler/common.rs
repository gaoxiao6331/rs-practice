// | 标题   | `#` `##` `###` |
// | 段落   | 普通文本           |
// | 粗体   | `**text**`     |
// | 行内代码 | `` `code` ``   |
// | 代码块  | ` ``` `        |
// | 无序列表 | `-`            |
// | 有序列表 | `1.`           |
// | 引用   | `>`            |
// | 链接   | `[text](url)`  |
// | 分割线  | `---`          |
// | 斜体  | `*text*`   |
// | 图片  | `![]()`    |
// | 删除线 | `~~text~~` |
// | 表格  | `\|`       |

use auto_lifetime::auto_lifetime;

#[auto_lifetime]
pub enum LineType {
    Heading { level: u64, text: &str },
    Paragraph { text: &str },
    CodeBlockStart { language: &str },
    CodeBlockEnd,
    UnorderedList { text: &str, indent: u64 },
    OrderedList { text: &str },
    Quote { text: &str },
    HorizontalRule,
    Image { alt: &str, url: &str },
    TableRow { text: &str },
    BlankLine,
}

#[auto_lifetime]
pub enum TextNode {
    BoldText { text: &str },
    ItalicText { text: &str },
    PlainText { text: &str },
    StrikethroughText { text: &str },
    InlineCode { text: &str },
}

#[auto_lifetime]
pub enum Node {
    // heading
    Heading {
        level: u64,
        text: &str,
    },
    // paragraph
    Paragraph {
        children: Node,
    },
    // text
    Text {
        children: Vec<Node>,
    },

    // code block
    CodeBlock {
        language: &str,
        code: &str,
    },

    ListItem {
        text: Option<TextNode>,
        children: Option<Vec<TextNode>>,
    },

    // unordered list
    UnorderedList {
        children: Vec<ListItem>,
    },

    // ordered list
    OrderedList {
        children: Vec<ListItem>,
    },
    // quote
    Quote {
        text: TextNode,
    },
    // horizontal rule
    HorizontalRule,
    // image
    Image {
        alt: String,
        url: String,
    },
    // table
    Table {
        children: Vec<TableRow>,
    },
    TableRow {
        children: Vec<TableCell>,
    },
    TableCell {
        text: String,
    },
}

pub struct Ast {
    children: Vec<Node>,
}
