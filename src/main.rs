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
        "a*",
        "b+",
        "c?",
        "a*?",
        "b+?",
        "c??",
        "x{3}",
        "y{2,}",
        "z{1,3}",
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
    use ast::{AnchorType, Quantifier, RegexNode};

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

    #[test]
    fn test_basic_quantifiers() {
        let test_cases = vec![
            (
                "a*",
                vec![RegexNode::new_literal('a').with_quantifier(Quantifier::ZeroOrMore { lazy: false })]
            ),
            (
                "b+",
                vec![RegexNode::new_literal('b').with_quantifier(Quantifier::OneOrMore { lazy: false })]
            ),
            (
                "c?",
                vec![RegexNode::new_literal('c').with_quantifier(Quantifier::ZeroOrOne { lazy: false })]
            ),
        ];

        for (pattern, expected) in test_cases {
            let mut parser = Parser::new(pattern);
            let result = parser.parse().unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_lazy_quantifiers() {
        let test_cases = vec![
            (
                "a*?",
                vec![RegexNode::new_literal('a').with_quantifier(Quantifier::ZeroOrMore { lazy: true })]
            ),
            (
                "b+?",
                vec![RegexNode::new_literal('b').with_quantifier(Quantifier::OneOrMore { lazy: true })]
            ),
            (
                "c??",
                vec![RegexNode::new_literal('c').with_quantifier(Quantifier::ZeroOrOne { lazy: true })]
            ),
        ];

        for (pattern, expected) in test_cases {
            let mut parser = Parser::new(pattern);
            let result = parser.parse().unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_curly_quantifiers() {
        let test_cases = vec![
            (
                "a{3}",
                vec![RegexNode::new_literal('a').with_quantifier(Quantifier::Exactly(3))]
            ),
            (
                "b{2,}",
                vec![RegexNode::new_literal('b').with_quantifier(Quantifier::AtLeast(2))]
            ),
            (
                "c{1,3}",
                vec![RegexNode::new_literal('c').with_quantifier(Quantifier::Range { min: 1, max: 3 })]
            ),
        ];

        for (pattern, expected) in test_cases {
            let mut parser = Parser::new(pattern);
            let result = parser.parse().unwrap();
            assert_eq!(result, expected);
        }
    }
}
