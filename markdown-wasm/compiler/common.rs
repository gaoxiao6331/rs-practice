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
pub mod ast {
    pub enum LineType {
        Heading { level: u64, text: &str },
        CodeBlockStart { language: &str },
        CodeBlockEnd,
        UnorderedList { text: &str, indent: u64 },
        OrderedList { text: &str },
        Quote { text: &str },
        HorizontalRule,
        TableRowLike { columns: Vec<&str> }, // 这里可能不是table，需要根据上下文判断
        BlankLine,
        Other{ text: &str },
    }

    pub struct ListItem {
        children: Vec<Inline>,
        nested: Vec<ListItem>,
    }

    pub struct TableCell {
        children: Vec<Inline>,
    }

    pub struct TableRow {
        cells: Vec<TableCell>,
    }

    pub enum Inline {
        Bold { children: Vec<Inline> },
        Italic { children: Vec<Inline> },
        Text { text: &str },
        Strikethrough { children: Vec<Inline> },
        InlineCode { text: &str },
        Image { alt: &str, url: &str },
        Link { children: Vec<Inline>, url: &str },
    }

    pub enum Block {
        // heading
        Heading {
            level: u64,
            text: &str,
        },
        // paragraph
        Paragraph {
            children: Vec<Inline>,
        },

        // code block
        CodeBlock {
            language: &str,
            code: &str,
        },

        // unordered list
        List {
            order: bool,
            children: Vec<ListItem>,
        },

        // quote
        Quote {
            children: Vec<Inline>,
        },
        // horizontal rule
        HorizontalRule,

        // table
        Table {
            children: Vec<TableRow>,
        },
    }

    pub struct Ast {
        children: Vec<Block>,
    }
}
