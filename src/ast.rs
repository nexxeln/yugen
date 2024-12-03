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
    CharacterType(CharacterTypeKind),
    UnicodeCategory {
        negated: bool,
        category: UnicodeCategoryKind,
    },
    Alternation(Vec<Vec<RegexNode>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CharacterTypeKind {
    Word,           // \w
    NotWord,        // \W
    Digit,          // \d
    NotDigit,       // \D
    Whitespace,     // \s
    NotWhitespace,  // \S
    EscapedChar(EscapedChar),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EscapedChar {
    Tab,            // \t
    NewLine,        // \n
    CarriageReturn, // \r
    FormFeed,       // \f
    VerticalTab,    // \v
    Null,           // \0
    Hex(u32),      // \xHH
    Unicode(u32),   // \u{H...}
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnicodeCategoryKind {
    Letter,              // \p{L}
    Number,              // \p{N}
    Punctuation,         // \p{P}
    Symbol,              // \p{S}
    Mark,                // \p{M}
    Separator,           // \p{Z}
    Other,               // \p{C}
    // Add more categories as needed
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

    pub fn new_character_type(kind: CharacterTypeKind) -> Self {
        RegexNode::CharacterType(kind)
    }

    pub fn new_unicode_category(category: UnicodeCategoryKind, negated: bool) -> Self {
        RegexNode::UnicodeCategory { category, negated }
    }

    pub fn new_alternation(alternatives: Vec<Vec<RegexNode>>) -> Self {
        RegexNode::Alternation(alternatives)
    }
} 