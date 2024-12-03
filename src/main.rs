mod ast;
mod parser;

use parser::Parser;

fn main() {
    let test_patterns = vec![
        "abc",
        "a.c",
        "[abc]",
        "[^xyz]",
        "^hello$",
        "\\bword\\b",
    ];

    for pattern in test_patterns {
        println!("Parsing pattern: {}", pattern);
        let mut parser = Parser::new(pattern);
        match parser.parse() {
            Ok(nodes) => println!("AST: {:?}\n", nodes),
            Err(e) => println!("Error: {:?}\n", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::{AnchorType, RegexNode};

    #[test]
    fn test_basic_parsing() {
        let mut parser = Parser::new("abc");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_literal('a'),
                RegexNode::new_literal('b'),
                RegexNode::new_literal('c'),
            ]
        );
    }

    #[test]
    fn test_character_class() {
        let mut parser = Parser::new("[abc]");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_char_class(vec!['a', 'b', 'c'], false)]
        );
    }

    #[test]
    fn test_anchors() {
        let mut parser = Parser::new("^abc$");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_anchor(AnchorType::Start),
                RegexNode::new_literal('a'),
                RegexNode::new_literal('b'),
                RegexNode::new_literal('c'),
                RegexNode::new_anchor(AnchorType::End),
            ]
        );
    }
}
