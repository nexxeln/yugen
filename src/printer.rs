use crate::ast::{RegexNode, GroupKind, Quantifier, CharacterTypeKind, EscapedChar, AnchorType};

pub struct Printer {
    use_unicode_escapes: bool,
}

impl Printer {
    pub fn new(use_unicode_escapes: bool) -> Self {
        Printer { use_unicode_escapes }
    }

    pub fn print(&self, ast: &[RegexNode]) -> String {
        ast.iter()
            .map(|node| self.print_node(node))
            .collect::<Vec<_>>()
            .join("")
    }

    fn print_node(&self, node: &RegexNode) -> String {
        match node {
            RegexNode::Literal(c) => self.print_char(*c),
            RegexNode::CharacterClass { negated, chars } => {
                let mut result = String::from("[");
                if *negated {
                    result.push('^');
                }
                result.push_str(
                    &chars
                        .iter()
                        .map(|c| self.print_char(*c))
                        .collect::<Vec<_>>()
                        .join(""),
                );
                result.push(']');
                result
            }
            RegexNode::Dot => ".".to_string(),
            RegexNode::Anchor(anchor_type) => match anchor_type {
                AnchorType::Start => "^".to_string(),
                AnchorType::End => "$".to_string(),
            },
            RegexNode::WordBoundary => "\\b".to_string(),
            RegexNode::Quantified { node, quantifier } => {
                format!("{}{}", self.print_node(node), self.print_quantifier(quantifier))
            }
            RegexNode::Group(kind, nodes) => {
                let contents = self.print(nodes);
                match kind {
                    GroupKind::Capturing(None) => format!("({})", contents),
                    GroupKind::Capturing(Some(name)) => format!("(?<{}>{})", name, contents),
                    GroupKind::NonCapturing => format!("(?:{})", contents),
                }
            }
            RegexNode::Alternation(alternatives) => alternatives
                .iter()
                .map(|alt| self.print(alt))
                .collect::<Vec<_>>()
                .join("|"),
            RegexNode::CharacterType(char_type) => match char_type {
                CharacterTypeKind::Word => "\\w".to_string(),
                CharacterTypeKind::NotWord => "\\W".to_string(),
                CharacterTypeKind::Digit => "\\d".to_string(),
                CharacterTypeKind::NotDigit => "\\D".to_string(),
                CharacterTypeKind::Whitespace => "\\s".to_string(),
                CharacterTypeKind::NotWhitespace => "\\S".to_string(),
                CharacterTypeKind::EscapedChar(esc) => self.print_escaped_char(esc),
            },
            // Add other cases as needed
            _ => String::new(),
        }
    }

    fn print_char(&self, c: char) -> String {
        if self.use_unicode_escapes {
            format!("\\u{{{:X}}}", c as u32)
        } else {
            c.to_string()
        }
    }

    fn print_quantifier(&self, quantifier: &Quantifier) -> String {
        match quantifier {
            Quantifier::ZeroOrMore { lazy } => if *lazy { "*?" } else { "*" }.to_string(),
            Quantifier::OneOrMore { lazy } => if *lazy { "+?" } else { "+" }.to_string(),
            Quantifier::ZeroOrOne { lazy } => if *lazy { "??" } else { "?" }.to_string(),
            Quantifier::Exactly(n) => format!("{{{}}}", n),
            Quantifier::AtLeast(n) => format!("{{{},}}", n),
            Quantifier::Range { min, max } => format!("{{{},{}}}", min, max),
        }
    }

    fn print_escaped_char(&self, escaped_char: &EscapedChar) -> String {
        match escaped_char {
            EscapedChar::Tab => "\\t".to_string(),
            EscapedChar::NewLine => "\\n".to_string(),
            EscapedChar::CarriageReturn => "\\r".to_string(),
            EscapedChar::FormFeed => "\\f".to_string(),
            EscapedChar::VerticalTab => "\\v".to_string(),
            EscapedChar::Null => "\\0".to_string(),
            EscapedChar::Hex(n) => format!("\\x{:02X}", n),
            EscapedChar::Unicode(n) => format!("\\u{{{:X}}}", n),
        }
    }
} 