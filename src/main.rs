mod ast;
mod parser;
mod printer;
mod obfuscator;
mod tests;

use parser::Parser;
use printer::Printer;
use obfuscator::Obfuscator;

fn main() {
    let pattern = "hello";
    println!("Original regex pattern: {}", pattern);
    
    // Parse the pattern into AST
    let mut parser = Parser::new(pattern);
    let ast = parser.parse().unwrap();
    
    // Obfuscate the AST
    let mut obfuscator = Obfuscator::new();
    let obfuscated_ast = obfuscator.obfuscate(ast);
    
    // Convert back to string with Unicode escapes
    let printer = Printer::new(true);
    let obfuscated_pattern = printer.print(&obfuscated_ast);
    
    println!("Obfuscated pattern: {}", obfuscated_pattern);
    
    // Print without Unicode escapes to verify it's equivalent
    let normal_printer = Printer::new(false);
    let normal_pattern = normal_printer.print(&obfuscated_ast);
    println!("Same pattern without escapes: {}", normal_pattern);
}
