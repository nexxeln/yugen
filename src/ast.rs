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
    Quantified {
        node: Box<RegexNode>,
        quantifier: Quantifier,
    },
    Group(GroupKind, Vec<RegexNode>),
    Backreference(BackreferenceKind),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupKind {
    Capturing(Option<String>), // None for unnamed, Some(name) for named groups
    NonCapturing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackreferenceKind {
    NumberBased(usize),     // \1, \2, etc.
    NameBased(String),      // \k<name>
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnchorType {
    Start, // ^
    End,   // $
}

#[derive(Debug, Clone, PartialEq)]
pub enum Quantifier {
    ZeroOrMore { lazy: bool },     // * or *?
    OneOrMore { lazy: bool },      // + or +?
    ZeroOrOne { lazy: bool },      // ? or ??
    Exactly(usize),                // {n}
    AtLeast(usize),                // {n,}
    Range { min: usize, max: usize }, // {n,m}
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

    pub fn with_quantifier(self, quantifier: Quantifier) -> Self {
        RegexNode::Quantified {
            node: Box::new(self),
            quantifier,
        }
    }

    pub fn new_group(kind: GroupKind, nodes: Vec<RegexNode>) -> Self {
        RegexNode::Group(kind, nodes)
    }

    pub fn new_backreference(kind: BackreferenceKind) -> Self {
        RegexNode::Backreference(kind)
    }
} 