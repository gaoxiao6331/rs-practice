use crate::markdown_wasm::compiler::common::ast::Inline;
use crate::markdown_wasm::compiler::common::ast::Inline::{Bold, Text, Image, Italic, Strikethrough};
/*
     pub enum Inline {
        Bold { children: Vec<Inline> }, **xx**
        Italic { children: Vec<Inline> }, *xx*
        Text { text: &str },
        Strikethrough { children: Vec<Inline> }, ~~xxx~~
        InlineCode { text: &str }, `xxx`
        Image { alt: &str, url: &str }, ![]()
        Link { children: Vec<Inline>, url: &str }, []()
    }
 */

// 给link和image公用的解析逻辑
fn handle_link_like(link_like: &str, link: bool) -> (Option<Inline<'_>>, usize) {

    let mut chars = link_like.chars().enumerate();

    // [
    if let Some((_, c)) = chars.next() {
        if c == '['{
            // ]
            if let Some((idx_right_square_bracket, _)) = chars.find(|(_, c)| *c == ']') {
                let round_bracket_str = &link_like[idx_right_square_bracket + 1..];
                let mut round_bracket_chars = round_bracket_str.chars().enumerate();
                // (
                if let Some ((idx_left_round_bracket, left_round_bracket_char)) = round_bracket_chars.next() {
                    if left_round_bracket_char == '(' {
                        // )
                        if let Some ((idx_right_round_bracket, _)) = round_bracket_chars.find(|(_, c)| *c == ')') {

                            let len = idx_right_round_bracket + 1;

                            let content = &link_like[1..idx_right_square_bracket];

                            // link需要解析富文本
                            let children = if link {
                                parse_inline(content)
                            } else { // image不解析
                                vec![
                                    Inline::Text { text: content },
                                ]
                            };

                            let url = &link_like[idx_left_round_bracket + 1..idx_right_round_bracket];

                            return (
                                Some(Inline::Link {
                                    url,
                                    children,
                                }),
                                len
                            );
                        }
                    }
                }
            }
        }
    }

    (Option::None, 0)
}

fn parse_inline(inline: &str) -> Vec<Inline<'_>> {

    let mut result = vec![];

    // 下一个要消费的字符串的起始位置
    let mut next_start_idx = 0;

    // 正在试探的字符的位置
    let mut testing_idx = 0;

    while testing_idx < inline.len() {

        let testing_str = &inline[testing_idx..];

        let mut chars = testing_str.chars().enumerate();

        // 是否匹配成功
        let mut matched = false;

        // TODO 这个first_idx有问题
        if let Some((first_idx, first_char)) = chars.next()  {

            match first_char {
                '*' => {
                    if let Some((i,c)) = chars.next() {

                        if c == '*' { // bold

                            let sub = &inline[i+1..];
                            // 找结束的**
                            if let Some(end_idx) = sub.find("**") {
                                let children_str = &sub[..end_idx];

                                let children = parse_inline(children_str);

                                result.push(Text {
                                    text: &inline[next_start_idx..first_idx]
                                });

                                result.push(Bold {
                                    children
                                });

                                next_start_idx = end_idx + 2;

                                matched = true;

                            } else {

                            }

                        } else { // italic
                            let sub = &inline[i+1..];
                            // 找结束的**
                            if let Some(end_idx) = sub.find("*") {
                                let children_str = &sub[..end_idx];

                                let children = parse_inline(children_str);

                                result.push(Text {
                                    text: &inline[next_start_idx..first_idx]
                                });

                                result.push(Italic {
                                    children
                                });

                                next_start_idx = end_idx + 1;

                                matched = true;

                            } else {

                            }
                        }

                    } else {

                    }
                },
                '~' => {
                    if let Some((i,c)) = chars.next() {

                        if c == '~' {

                            let sub = &inline[i+1..];
                            // 找结束的**
                            if let Some(end_idx) = sub.find("~~") {
                                let children_str = &sub[..end_idx];

                                let children = parse_inline(children_str);

                                result.push(Text {
                                    text: &inline[next_start_idx..first_idx]
                                });

                                result.push(Strikethrough {
                                    children
                                });

                                next_start_idx = end_idx + 2;

                                matched = true;

                            } else {

                            }

                        } else {

                        }
                    } else {

                    }
                },
                '`' => {
                    let sub = &inline[first_idx+1..];
                    // 找结束的**
                    if let Some(end_idx) = sub.find("`") {
                        let children_str = &sub[..end_idx];

                        result.push(Text {
                            text: &inline[next_start_idx..first_idx]
                        });

                        result.push( {
                            Inline::InlineCode {
                                text: children_str
                            }
                        });

                        next_start_idx = end_idx + 1;

                        matched = true;

                    } else {

                    }
                },
                '!' => {

                    let (link_result, len) = handle_link_like(&testing_str[1..], false);

                    if let Some(link) = link_result {

                        if let Inline::Link {
                            children,
                            url
                        } = link {

                            if children.len() == 1  {

                                if let Some(f) = children.first() {

                                    if let Inline::Text { text } = f {
                                        matched = true;

                                        result.push(Inline::Image {
                                            alt: text,
                                            url,
                                        });

                                        next_start_idx += (len + 1);
                                    }
                                }
                            }

                        }

                    }
                },
                '[' => {
                    let (link_result, len) = handle_link_like(testing_str, true);

                    if let Some(link) = link_result {
                        matched = true;

                        result.push(link);

                        next_start_idx += len;
                    }
                },
                _ => {

                },
            }

            // 如果没有匹配成功，则就行测试下一个字符
            if !matched {
                testing_idx += 1;
            } else {
                testing_idx = next_start_idx;
            }
        }
    }

    if next_start_idx < inline.len() {
        result.push( Text {
            text: &inline[next_start_idx..]
        })
    }

    result
}