use super::common::LineType;



fn scan_line(md: &str) -> Vec<LineType<'_>> {
    let mut tokens = vec![];

    let lines = md.lines();
    
    for line in lines {
        
        let line = line.trim();
        
        if line.is_empty() {
            tokens.push(LineType::BlankLine);
            continue;
        }

        
        
    }
    tokens
}
