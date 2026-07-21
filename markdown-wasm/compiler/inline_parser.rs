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


fn handle_link(link_line: &str) -> (Vec<Inline>, usize) {
    let mut result = vec![];
    let mut next_idx = 0;

    let mut chars = link_line.chars().enumerate();

    if let Some((idx, c)) = chars.next() {

    }

    (result, next_idx)
}

fn handle_line(line: &str) -> Vec<Inline> {

    let mut result = vec![];

    // 下一个要消费的字符串的起始位置
    let mut next_start_idx = 0;

    // 正在试探的字符的位置
    let mut testing_idx = 0;

    while testing_idx < line.len() {

        let mut chars = (&line[testing_idx..]).chars().enumerate();

        // 是否匹配成功
        let mut matched = false;

        if let Some((first_idx, first_char)) = chars.next()  {

            match first_char {
                '*' => {
                    if let Some((i,c)) = chars.next() {

                        if c == '*' { // bold

                            let sub = &line[i+1..];
                            // 找结束的**
                            if let Some(end_idx) = sub.find("**") {
                                let children_str = &sub[..end_idx];

                                let children = handle_line(children_str);

                                result.push(Text {
                                    text: &line[next_start_idx..first_idx]
                                });

                                result.push(Bold {
                                    children
                                });

                                next_start_idx = end_idx + 2;

                                matched = true;

                            } else {

                            }

                        } else { // italic
                            let sub = &line[i+1..];
                            // 找结束的**
                            if let Some(end_idx) = sub.find("*") {
                                let children_str = &sub[..end_idx];

                                let children = handle_line(children_str);

                                result.push(Text {
                                    text: &line[next_start_idx..first_idx]
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

                            let sub = &line[i+1..];
                            // 找结束的**
                            if let Some(end_idx) = sub.find("~~") {
                                let children_str = &sub[..end_idx];

                                let children = handle_line(children_str);

                                result.push(Text {
                                    text: &line[next_start_idx..first_idx]
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
                    let sub = &line[first_idx+1..];
                    // 找结束的**
                    if let Some(end_idx) = sub.find("`") {
                        let children_str = &sub[..end_idx];

                        result.push(Text {
                            text: &line[next_start_idx..first_idx]
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

                },
                '[' => {

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

    if next_start_idx < line.len() {
        result.push( Text {
            text: &line[next_start_idx..]
        })
    }

    result
}

fn parse_inline(inline: &str) -> Vec<Inline> {
    let mut inlines = vec![];

    // TODO 并行解析
    for line in inline.lines() {

        let mut buffer = String::new();




    }
    inlines
}