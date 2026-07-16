use std::collections::HashMap;
use super::common::ast::LineType;

fn scan_line(md: &str) -> Vec<LineType<'_>> {

    let mut tokens = vec![];

    let lines = md.lines();

    // TODO 这里可以多线程
    for line in lines {
        let line = line.trim();

        if line.is_empty() {
            tokens.push(LineType::BlankLine);
            continue;
        }

        let mut  chars = line.char_indices();
        if let Some((_, first)) = chars.next() {
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
            let line_type = match first {
                // h
                '#' => {
                    // 多个# + 一个空格
                    let mut hash_count = 1;
                    let mut space = false;

                    let mut idx: usize = 0;

                    while let Some ((i, c)) = chars.next() {

                        idx = i;

                        if c == ' ' {
                            space = true;
                            break;
                        }

                        if c != '#' {
                            break;
                        } else {
                            hash_count += 1;
                        }
                    }
                    // 如果没有space认定为普通文本
                    let res = if space {
                        let text = &line[idx..];
                        LineType::Heading { text, level: hash_count }
                    } else {
                        LineType::Other { text: line }
                    };

                    res
                },
                // hr ul
                '-' => {
                    // 如果有3个-，且只有-和空格，则是hr
                    let mut without_space = line.chars().filter(|c| *c != ' ');
                    let is_hr = without_space.all(|c| c == '-') && without_space.count() >= 3;
                    if is_hr {
                        LineType::HorizontalRule
                    } else {
                        if let Some(text) = line.strip_prefix("- ") {
                            LineType::UnorderedList {
                                indent: 0,
                                text,
                            }
                        } else {
                            LineType::Other { text: line }
                        }
                    }
                },
                ' ' => {
                    // unordered list
                    // TODO
                    LineType::Other { text: line }
                },
                // quote 不支持嵌套
                '>' => {
                    // 如果是空格，则认为是引用
                    let res = if let Some((i, c)) = chars.next() {
                        if c == ' ' {
                            LineType::Quote {
                                text: &line[i..],
                            }
                        } else {
                            LineType::Other { text: line }
                        }
                    } else {
                        LineType::Other { text: line }
                    };

                    res
                },
                // table
                '|' => {
                    LineType::Other { text: line }
                },
                // ol
                '0'..='9' => {
                    let mut n = first;
                    // 去掉后续的数字
                    while let Some((_, c)) = chars.next() {
                        if c.is_digit(10) {
                            continue
                        } else {
                            n = c;
                        }
                    }
                    // 判断是否是'.', 如果是，则是ol，不是的话，认为是普通文本
                    if n == '.' {
                        let text = if let Some((i, _)) = chars.next() {
                            &line[i..]
                        } else {
                            ""
                        };
                        LineType::OrderedList { text }
                    } else {
                        LineType::Other { text: line }
                    }
                },
                _ => {
                    LineType::Other { text: line }
                }
            };

            tokens.push(line_type);
        }

    }
    tokens
}
