use crate::markdown_wasm::compiler::common::ast::Inline;


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
fn parse_inline(inline: &str) -> Vec<Inline> {
    let mut inlines = vec![];

    // TODO 并行解析
    for line in inline.lines() {
        for (i, c) in line.chars().enumerate() {
            match c {
                '*' => {

                },
                '~' => {

                },
                '`' => {

                },
                '!' => {

                },
                '[' => {

                },
                _ => {

                },
            }
        }
    }
    inlines
}