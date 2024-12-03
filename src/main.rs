mod ast;
mod parser;
mod tests;

use parser::Parser;

fn main() {
    let pattern = "(?i:foo|bar)baz";
    println!("Parsing regex pattern: {}", pattern);
    
    let mut parser = Parser::new(pattern);
    match parser.parse() {
        Ok(ast) => println!("AST: {:#?}", ast),
        Err(e) => println!("Error: {:?}", e),
    }
}
