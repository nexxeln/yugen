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
        "(foo)",
        "(?:bar)",
        "(?<name>baz)",
        "(test)\\1",
        "(?<num>\\d+)\\k<num>",
        "\\w+",
        "\\d+",
        "\\s*",
        "\\p{L}+",
        "\\P{N}*",
        "\\n",
        "\\t",
        "\\x20",
        "\\u{1F600}",
        "cat|dog",
        "foo|bar|baz",
        "(cat|dog)+",
        "a(b|c)d",
        "\\w+|\\d+",
        "(?=foo)bar",
        "(?!foo)bar",
        "(?<=foo)bar",
        "(?<!foo)bar",
        "\\w+(?=\\d)",
        "(?<!\\s)\\w+",
        "foo(?!bar|baz)",
        "(?i)abc",
        "(?m)^abc$",
        "(?s)a.c",
        "(?i:foo)bar",
        "(?im)abc",
        "(?i)foo(?-i)bar",
        "a(?i)b(?-i)c",
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
    use ast::{AnchorType, BackreferenceKind, CharacterTypeKind, EscapedChar, GroupKind, LookaroundKind, Quantifier, RegexNode, UnicodeCategoryKind, RegexFlags};

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

    #[test]
    fn test_capturing_group() {
        let mut parser = Parser::new("(abc)");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_group(
                GroupKind::Capturing(None),
                vec![
                    RegexNode::new_literal('a'),
                    RegexNode::new_literal('b'),
                    RegexNode::new_literal('c'),
                ]
            )]
        );
    }

    #[test]
    fn test_non_capturing_group() {
        let mut parser = Parser::new("(?:abc)");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_group(
                GroupKind::NonCapturing,
                vec![
                    RegexNode::new_literal('a'),
                    RegexNode::new_literal('b'),
                    RegexNode::new_literal('c'),
                ]
            )]
        );
    }

    #[test]
    fn test_named_group() {
        let mut parser = Parser::new("(?<test>abc)");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_group(
                GroupKind::Capturing(Some("test".to_string())),
                vec![
                    RegexNode::new_literal('a'),
                    RegexNode::new_literal('b'),
                    RegexNode::new_literal('c'),
                ]
            )]
        );
    }

    #[test]
    fn test_backreference_number() {
        let mut parser = Parser::new("(a)\\1");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_group(
                    GroupKind::Capturing(None),
                    vec![RegexNode::new_literal('a')]
                ),
                RegexNode::new_backreference(BackreferenceKind::NumberBased(1))
            ]
        );
    }

    #[test]
    fn test_backreference_name() {
        let mut parser = Parser::new("(?<test>a)\\k<test>");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_group(
                    GroupKind::Capturing(Some("test".to_string())),
                    vec![RegexNode::new_literal('a')]
                ),
                RegexNode::new_backreference(BackreferenceKind::NameBased("test".to_string()))
            ]
        );
    }

    #[test]
    fn test_nested_groups() {
        let mut parser = Parser::new("(a(?:b(c)))");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_group(
                GroupKind::Capturing(None),
                vec![
                    RegexNode::new_literal('a'),
                    RegexNode::new_group(
                        GroupKind::NonCapturing,
                        vec![
                            RegexNode::new_literal('b'),
                            RegexNode::new_group(
                                GroupKind::Capturing(None),
                                vec![RegexNode::new_literal('c')]
                            )
                        ]
                    )
                ]
            )]
        );
    }

    #[test]
    fn test_group_with_quantifier() {
        let mut parser = Parser::new("(abc)+");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_group(
                GroupKind::Capturing(None),
                vec![
                    RegexNode::new_literal('a'),
                    RegexNode::new_literal('b'),
                    RegexNode::new_literal('c'),
                ]
            ).with_quantifier(Quantifier::OneOrMore { lazy: false })]
        );
    }

    #[test]
    fn test_character_types() {
        let test_cases = vec![
            (
                "\\w",
                vec![RegexNode::new_character_type(CharacterTypeKind::Word)]
            ),
            (
                "\\W",
                vec![RegexNode::new_character_type(CharacterTypeKind::NotWord)]
            ),
            (
                "\\d",
                vec![RegexNode::new_character_type(CharacterTypeKind::Digit)]
            ),
            (
                "\\D",
                vec![RegexNode::new_character_type(CharacterTypeKind::NotDigit)]
            ),
            (
                "\\s",
                vec![RegexNode::new_character_type(CharacterTypeKind::Whitespace)]
            ),
            (
                "\\S",
                vec![RegexNode::new_character_type(CharacterTypeKind::NotWhitespace)]
            ),
        ];

        for (pattern, expected) in test_cases {
            let mut parser = Parser::new(pattern);
            let result = parser.parse().unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_escaped_chars() {
        let test_cases = vec![
            (
                "\\n",
                vec![RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::NewLine
                ))]
            ),
            (
                "\\t",
                vec![RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::Tab
                ))]
            ),
            (
                "\\r",
                vec![RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::CarriageReturn
                ))]
            ),
            (
                "\\x20",
                vec![RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::Hex(0x20)
                ))]
            ),
            (
                "\\u{1F600}",
                vec![RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::Unicode(0x1F600)
                ))]
            ),
        ];

        for (pattern, expected) in test_cases {
            let mut parser = Parser::new(pattern);
            let result = parser.parse().unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_unicode_categories() {
        let test_cases = vec![
            (
                "\\p{L}",
                vec![RegexNode::new_unicode_category(UnicodeCategoryKind::Letter, false)]
            ),
            (
                "\\P{N}",
                vec![RegexNode::new_unicode_category(UnicodeCategoryKind::Number, true)]
            ),
            (
                "\\p{P}",
                vec![RegexNode::new_unicode_category(UnicodeCategoryKind::Punctuation, false)]
            ),
        ];

        for (pattern, expected) in test_cases {
            let mut parser = Parser::new(pattern);
            let result = parser.parse().unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_combined_patterns() {
        let mut parser = Parser::new("\\w+\\s*\\p{L}+");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_character_type(CharacterTypeKind::Word)
                    .with_quantifier(Quantifier::OneOrMore { lazy: false }),
                RegexNode::new_character_type(CharacterTypeKind::Whitespace)
                    .with_quantifier(Quantifier::ZeroOrMore { lazy: false }),
                RegexNode::new_unicode_category(UnicodeCategoryKind::Letter, false)
                    .with_quantifier(Quantifier::OneOrMore { lazy: false }),
            ]
        );
    }

    #[test]
    fn test_basic_alternation() {
        let mut parser = Parser::new("cat|dog");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_alternation(vec![
                vec![
                    RegexNode::new_literal('c'),
                    RegexNode::new_literal('a'),
                    RegexNode::new_literal('t'),
                ],
                vec![
                    RegexNode::new_literal('d'),
                    RegexNode::new_literal('o'),
                    RegexNode::new_literal('g'),
                ],
            ])]
        );
    }

    #[test]
    fn test_multiple_alternation() {
        let mut parser = Parser::new("foo|bar|baz");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_alternation(vec![
                vec![
                    RegexNode::new_literal('f'),
                    RegexNode::new_literal('o'),
                    RegexNode::new_literal('o'),
                ],
                vec![
                    RegexNode::new_literal('b'),
                    RegexNode::new_literal('a'),
                    RegexNode::new_literal('r'),
                ],
                vec![
                    RegexNode::new_literal('b'),
                    RegexNode::new_literal('a'),
                    RegexNode::new_literal('z'),
                ],
            ])]
        );
    }

    #[test]
    fn test_alternation_in_group() {
        let mut parser = Parser::new("(cat|dog)");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_group(
                GroupKind::Capturing(None),
                vec![RegexNode::new_alternation(vec![
                    vec![
                        RegexNode::new_literal('c'),
                        RegexNode::new_literal('a'),
                        RegexNode::new_literal('t'),
                    ],
                    vec![
                        RegexNode::new_literal('d'),
                        RegexNode::new_literal('o'),
                        RegexNode::new_literal('g'),
                    ],
                ])],
            )]
        );
    }

    #[test]
    fn test_alternation_with_quantifier() {
        let mut parser = Parser::new("(cat|dog)+");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_group(
                GroupKind::Capturing(None),
                vec![RegexNode::new_alternation(vec![
                    vec![
                        RegexNode::new_literal('c'),
                        RegexNode::new_literal('a'),
                        RegexNode::new_literal('t'),
                    ],
                    vec![
                        RegexNode::new_literal('d'),
                        RegexNode::new_literal('o'),
                        RegexNode::new_literal('g'),
                    ],
                ])],
            ).with_quantifier(Quantifier::OneOrMore { lazy: false })]
        );
    }

    #[test]
    fn test_alternation_with_character_types() {
        let mut parser = Parser::new("\\w+|\\d+");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![RegexNode::new_alternation(vec![
                vec![RegexNode::new_character_type(CharacterTypeKind::Word)
                    .with_quantifier(Quantifier::OneOrMore { lazy: false })],
                vec![RegexNode::new_character_type(CharacterTypeKind::Digit)
                    .with_quantifier(Quantifier::OneOrMore { lazy: false })],
            ])]
        );
    }

    #[test]
    fn test_alternation_with_surrounding_context() {
        let mut parser = Parser::new("a(b|c)d");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_literal('a'),
                RegexNode::new_group(
                    GroupKind::Capturing(None),
                    vec![RegexNode::new_alternation(vec![
                        vec![RegexNode::new_literal('b')],
                        vec![RegexNode::new_literal('c')],
                    ])],
                ),
                RegexNode::new_literal('d'),
            ]
        );
    }

    #[test]
    #[should_panic]
    fn test_empty_alternation() {
        let mut parser = Parser::new("a||b");
        parser.parse().unwrap();
    }

    #[test]
    fn test_positive_lookahead() {
        let mut parser = Parser::new("(?=foo)bar");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_lookaround(
                    LookaroundKind::PositiveLookahead,
                    vec![
                        RegexNode::new_literal('f'),
                        RegexNode::new_literal('o'),
                        RegexNode::new_literal('o'),
                    ],
                ),
                RegexNode::new_literal('b'),
                RegexNode::new_literal('a'),
                RegexNode::new_literal('r'),
            ]
        );
    }

    #[test]
    fn test_negative_lookahead() {
        let mut parser = Parser::new("(?!foo)bar");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_lookaround(
                    LookaroundKind::NegativeLookahead,
                    vec![
                        RegexNode::new_literal('f'),
                        RegexNode::new_literal('o'),
                        RegexNode::new_literal('o'),
                    ],
                ),
                RegexNode::new_literal('b'),
                RegexNode::new_literal('a'),
                RegexNode::new_literal('r'),
            ]
        );
    }

    #[test]
    fn test_positive_lookbehind() {
        let mut parser = Parser::new("(?<=foo)bar");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_lookaround(
                    LookaroundKind::PositiveLookbehind,
                    vec![
                        RegexNode::new_literal('f'),
                        RegexNode::new_literal('o'),
                        RegexNode::new_literal('o'),
                    ],
                ),
                RegexNode::new_literal('b'),
                RegexNode::new_literal('a'),
                RegexNode::new_literal('r'),
            ]
        );
    }

    #[test]
    fn test_negative_lookbehind() {
        let mut parser = Parser::new("(?<!foo)bar");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_lookaround(
                    LookaroundKind::NegativeLookbehind,
                    vec![
                        RegexNode::new_literal('f'),
                        RegexNode::new_literal('o'),
                        RegexNode::new_literal('o'),
                    ],
                ),
                RegexNode::new_literal('b'),
                RegexNode::new_literal('a'),
                RegexNode::new_literal('r'),
            ]
        );
    }

    #[test]
    fn test_lookaround_with_alternation() {
        let mut parser = Parser::new("foo(?!bar|baz)");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_literal('f'),
                RegexNode::new_literal('o'),
                RegexNode::new_literal('o'),
                RegexNode::new_lookaround(
                    LookaroundKind::NegativeLookahead,
                    vec![RegexNode::new_alternation(vec![
                        vec![
                            RegexNode::new_literal('b'),
                            RegexNode::new_literal('a'),
                            RegexNode::new_literal('r'),
                        ],
                        vec![
                            RegexNode::new_literal('b'),
                            RegexNode::new_literal('a'),
                            RegexNode::new_literal('z'),
                        ],
                    ])],
                ),
            ]
        );
    }

    #[test]
    fn test_lookaround_with_character_types() {
        let mut parser = Parser::new("\\w+(?=\\d)");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            vec![
                RegexNode::new_character_type(CharacterTypeKind::Word)
                    .with_quantifier(Quantifier::OneOrMore { lazy: false }),
                RegexNode::new_lookaround(
                    LookaroundKind::PositiveLookahead,
                    vec![RegexNode::new_character_type(CharacterTypeKind::Digit)],
                ),
            ]
        );
    }

    #[test]
    fn test_basic_flag() {
        let mut parser = Parser::new("(?i)abc");
        let result = parser.parse().unwrap();
        let mut flags = RegexFlags::new();
        flags.case_insensitive = true;
        assert_eq!(
            result,
            vec![RegexNode::new_flag_set(
                flags,
                vec![
                    RegexNode::new_literal('a'),
                    RegexNode::new_literal('b'),
                    RegexNode::new_literal('c'),
                ],
            )]
        );
    }

    #[test]
    fn test_multiple_flags() {
        let mut parser = Parser::new("(?im)abc");
        let result = parser.parse().unwrap();
        let mut flags = RegexFlags::new();
        flags.case_insensitive = true;
        flags.multiline = true;
        assert_eq!(
            result,
            vec![RegexNode::new_flag_set(
                flags,
                vec![
                    RegexNode::new_literal('a'),
                    RegexNode::new_literal('b'),
                    RegexNode::new_literal('c'),
                ],
            )]
        );
    }

    #[test]
    fn test_scoped_flags() {
        let mut parser = Parser::new("(?i:foo)bar");
        let result = parser.parse().unwrap();
        let mut flags = RegexFlags::new();
        flags.case_insensitive = true;
        assert_eq!(
            result,
            vec![
                RegexNode::new_flag_set(
                    flags,
                    vec![
                        RegexNode::new_literal('f'),
                        RegexNode::new_literal('o'),
                        RegexNode::new_literal('o'),
                    ],
                ),
                RegexNode::new_literal('b'),
                RegexNode::new_literal('a'),
                RegexNode::new_literal('r'),
            ]
        );
    }

    #[test]
    fn test_flag_with_anchors() {
        let mut parser = Parser::new("(?m)^abc$");
        let result = parser.parse().unwrap();
        let mut flags = RegexFlags::new();
        flags.multiline = true;
        assert_eq!(
            result,
            vec![RegexNode::new_flag_set(
                flags,
                vec![
                    RegexNode::new_anchor(AnchorType::Start),
                    RegexNode::new_literal('a'),
                    RegexNode::new_literal('b'),
                    RegexNode::new_literal('c'),
                    RegexNode::new_anchor(AnchorType::End),
                ],
            )]
        );
    }

    #[test]
    fn test_flag_with_dot() {
        let mut parser = Parser::new("(?s)a.c");
        let result = parser.parse().unwrap();
        let mut flags = RegexFlags::new();
        flags.dot_all = true;
        assert_eq!(
            result,
            vec![RegexNode::new_flag_set(
                flags,
                vec![
                    RegexNode::new_literal('a'),
                    RegexNode::Dot,
                    RegexNode::new_literal('c'),
                ],
            )]
        );
    }

    #[test]
    fn test_flag_with_alternation() {
        let mut parser = Parser::new("(?i:foo|bar)baz");
        let result = parser.parse().unwrap();
        let mut flags = RegexFlags::new();
        flags.case_insensitive = true;
        assert_eq!(
            result,
            vec![
                RegexNode::new_flag_set(
                    flags,
                    vec![RegexNode::new_alternation(vec![
                        vec![
                            RegexNode::new_literal('f'),
                            RegexNode::new_literal('o'),
                            RegexNode::new_literal('o'),
                        ],
                        vec![
                            RegexNode::new_literal('b'),
                            RegexNode::new_literal('a'),
                            RegexNode::new_literal('r'),
                        ],
                    ])],
                ),
                RegexNode::new_literal('b'),
                RegexNode::new_literal('a'),
                RegexNode::new_literal('z'),
            ]
        );
    }
}
