mod ast;
mod parser;
mod printer;
mod obfuscator;
mod tests;

use parser::Parser;
use printer::Printer;
use obfuscator::Obfuscator;

fn main() {
    let test_patterns = vec![
        "hello",           // Simple literals
        "[abc]",          // Basic character class
        "[a-z]",          // Character class with range
        "foo[bar]baz",    // Character class in context
        "[^abc]",         // Negated character class (will be preserved)
    ];

    for pattern in test_patterns {
        println!("\nProcessing pattern: {}", pattern);
        
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
}
