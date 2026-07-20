use crate::markdown_wasm::compiler::common::ast::{Ast, LineType};

fn parse_block(lines: Vec<LineType>) -> Ast {
    let mut ast = Ast {
        children: vec![]
    };

    ast
}