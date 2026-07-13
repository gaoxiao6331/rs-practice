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

/// 用 `#[auto_lifetime]` 自动给所有 `&str` 引用打上同一个生命周期 `'a`，
/// 免去手动写 `LineType<'a>` / `&'a str` 的样板代码。
#[auto_lifetime]
pub enum LineType {
    Heading { level: u64, text: &str },
    Paragraph { text: &str },
    CodeBlockStart { language: &str },
    CodeBlockEnd,
    UnorderedList { text: &str, indent: u64 },
    OrderedList { text: &str },
    Quote { text: &str },
    HorizentalRule,
    Image { alt: &str, url: &str },
    TableRow { text: &str },
    BlankLine,
}

pub enum Node {}

pub struct Ast {
    children: Vec<Node>,
}
