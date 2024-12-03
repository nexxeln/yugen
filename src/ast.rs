#[derive(Debug, Clone, PartialEq)]
pub enum RegexNode {
    Literal(char),
    CharacterClass {
        negated: bool,
        chars: Vec<char>,
    },
    Dot,
    Anchor(AnchorType),
    WordBoundary,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnchorType {
    Start, // ^
    End,   // $
}

impl RegexNode {
    pub fn new_literal(c: char) -> Self {
        RegexNode::Literal(c)
    }

    pub fn new_char_class(chars: Vec<char>, negated: bool) -> Self {
        RegexNode::CharacterClass { chars, negated }
    }

    pub fn new_anchor(anchor_type: AnchorType) -> Self {
        RegexNode::Anchor(anchor_type)
    }
} 