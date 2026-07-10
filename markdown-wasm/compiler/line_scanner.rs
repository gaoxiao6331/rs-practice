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

enum LineType {
    Heading(&str),
    Text(&str),
    Bold(&str),
    InlineCode(&str),
    CodeBlockStart(&str),
    CodeBlockEnd(&str),
    UnorderedList(&str, u64),
    OrderedList(&str),
    Quote(&str),
    Link(&str),
    HorizentalRule(&str),
    Italic(&str),
    Image(&str),
    Strikethrouth(&str),
    Table(&str),
}
