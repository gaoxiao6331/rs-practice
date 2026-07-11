use wasm_bindgen::prelude::*;

mod compiler;
use compiler::md2html::generate_html;

#[wasm_bindgen]
pub fn parse_markdown(input: &str) -> String {
    generate_html(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_placeholder_html() {
        let html = parse_markdown("# Hello Rust");
        assert!(html.contains("TODO"));
    }
}
