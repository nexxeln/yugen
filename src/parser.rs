use crate::ast::{
    AnchorType, BackreferenceKind, CharacterTypeKind, EscapedChar, GroupKind, LookaroundKind,
    Quantifier, RegexNode, UnicodeCategoryKind,
};

pub struct Parser {
    input: Vec<char>,
    position: usize,
    group_count: usize,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEndOfInput,
    UnexpectedCharacter(char),
    UnclosedCharacterClass,
    InvalidQuantifier,
    InvalidNumber,
    UnclosedGroup,
    InvalidGroupSyntax,
    InvalidBackreference,
    InvalidGroupName,
    InvalidUnicodeCategory,
    InvalidHexNumber,
    InvalidUnicodeValue,
    EmptyAlternation,
    InvalidLookaround,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Parser {
            input: input.chars().collect(),
            position: 0,
            group_count: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<RegexNode>, ParseError> {
        self.parse_alternation()
    }

    fn parse_alternation(&mut self) -> Result<Vec<RegexNode>, ParseError> {
        let mut alternatives = vec![Vec::new()];
        
        while !self.is_eof() {
            if self.current() == '|' {
                self.advance();
                alternatives.push(Vec::new());
                continue;
            }

            // Stop parsing alternation when we hit a closing parenthesis
            if self.current() == ')' {
                break;
            }

            let node = self.parse_node()?;
            if let Some(current_alt) = alternatives.last_mut() {
                current_alt.push(node);
            }
        }

        // If we have multiple alternatives, wrap them in an Alternation node
        if alternatives.len() > 1 {
            // Check for empty alternatives
            if alternatives.iter().any(|alt| alt.is_empty()) {
                return Err(ParseError::EmptyAlternation);
            }
            Ok(vec![RegexNode::new_alternation(alternatives)])
        } else {
            // If we only have one alternative, return it directly
            Ok(alternatives.into_iter().next().unwrap())
        }
    }

    fn parse_node(&mut self) -> Result<RegexNode, ParseError> {
        if self.is_eof() {
            return Err(ParseError::UnexpectedEndOfInput);
        }

        let node = match self.current() {
            '.' => {
                self.advance();
                RegexNode::Dot
            }
            '^' => {
                self.advance();
                RegexNode::new_anchor(AnchorType::Start)
            }
            '$' => {
                self.advance();
                RegexNode::new_anchor(AnchorType::End)
            }
            '\\' => {
                self.advance();
                self.parse_escape()?
            }
            '[' => self.parse_character_class()?,
            '(' => self.parse_group()?,
            c => {
                self.advance();
                RegexNode::new_literal(c)
            }
        };

        if !self.is_eof() {
            if let Some(quantifier) = self.try_parse_quantifier()? {
                return Ok(node.with_quantifier(quantifier));
            }
        }

        Ok(node)
    }

    fn try_parse_quantifier(&mut self) -> Result<Option<Quantifier>, ParseError> {
        if self.is_eof() {
            return Ok(None);
        }

        let quantifier = match self.current() {
            '*' => {
                self.advance();
                let lazy = self.check_lazy();
                Some(Quantifier::ZeroOrMore { lazy })
            }
            '+' => {
                self.advance();
                let lazy = self.check_lazy();
                Some(Quantifier::OneOrMore { lazy })
            }
            '?' => {
                self.advance();
                let lazy = self.check_lazy();
                Some(Quantifier::ZeroOrOne { lazy })
            }
            '{' => {
                self.advance();
                Some(self.parse_curly_quantifier()?)
            }
            _ => None,
        };

        Ok(quantifier)
    }

    fn check_lazy(&mut self) -> bool {
        if !self.is_eof() && self.current() == '?' {
            self.advance();
            true
        } else {
            false
        }
    }

    fn parse_curly_quantifier(&mut self) -> Result<Quantifier, ParseError> {
        let mut num_str = String::new();
        
        while !self.is_eof() && self.current().is_ascii_digit() {
            num_str.push(self.current());
            self.advance();
        }
        
        let n = num_str.parse::<usize>()
            .map_err(|_| ParseError::InvalidNumber)?;

        if self.is_eof() {
            return Err(ParseError::UnexpectedEndOfInput);
        }

        match self.current() {
            '}' => {
                self.advance();
                Ok(Quantifier::Exactly(n))
            }
            ',' => {
                self.advance();
                if self.is_eof() {
                    return Err(ParseError::UnexpectedEndOfInput);
                }

                if self.current() == '}' {
                    self.advance();
                    Ok(Quantifier::AtLeast(n))
                } else {
                    let mut max_str = String::new();
                    while !self.is_eof() && self.current().is_ascii_digit() {
                        max_str.push(self.current());
                        self.advance();
                    }

                    if self.is_eof() || self.current() != '}' {
                        return Err(ParseError::InvalidQuantifier);
                    }
                    self.advance();

                    let max = max_str.parse::<usize>()
                        .map_err(|_| ParseError::InvalidNumber)?;
                    
                    Ok(Quantifier::Range { min: n, max })
                }
            }
            _ => Err(ParseError::InvalidQuantifier),
        }
    }

    fn parse_character_class(&mut self) -> Result<RegexNode, ParseError> {
        self.advance(); // consume '['
        let negated = if self.current() == '^' {
            self.advance();
            true
        } else {
            false
        };

        let mut chars = Vec::new();
        while !self.is_eof() && self.current() != ']' {
            if self.current() == '\\' {
                self.advance();
                if self.is_eof() {
                    return Err(ParseError::UnexpectedEndOfInput);
                }
                chars.push(self.current());
                self.advance();
            } else {
                chars.push(self.current());
                self.advance();
            }
        }

        if self.is_eof() {
            return Err(ParseError::UnclosedCharacterClass);
        }

        self.advance(); // consume ']'
        Ok(RegexNode::new_char_class(chars, negated))
    }

    fn parse_group(&mut self) -> Result<RegexNode, ParseError> {
        self.advance(); // consume '('
        
        if self.check_char('?') {
            self.advance();
            match self.current() {
                ':' => {
                    self.advance();
                    let nodes = self.parse_alternation()?;
                    if self.is_eof() || self.current() != ')' {
                        return Err(ParseError::UnclosedGroup);
                    }
                    self.advance();
                    Ok(RegexNode::new_group(GroupKind::NonCapturing, nodes))
                }
                '<' => {
                    self.advance();
                    if self.check_char('=') || self.check_char('!') {
                        // Lookbehind
                        let negative = self.current() == '!';
                        self.advance();
                        let nodes = self.parse_alternation()?;
                        if self.is_eof() || self.current() != ')' {
                            return Err(ParseError::UnclosedGroup);
                        }
                        self.advance();
                        Ok(RegexNode::new_lookaround(
                            if negative {
                                LookaroundKind::NegativeLookbehind
                            } else {
                                LookaroundKind::PositiveLookbehind
                            },
                            nodes,
                        ))
                    } else {
                        // Named capturing group
                        let name = self.parse_group_name()?;
                        let nodes = self.parse_alternation()?;
                        if self.is_eof() || self.current() != ')' {
                            return Err(ParseError::UnclosedGroup);
                        }
                        self.advance();
                        Ok(RegexNode::new_group(GroupKind::Capturing(Some(name)), nodes))
                    }
                }
                '=' | '!' => {
                    // Lookahead
                    let negative = self.current() == '!';
                    self.advance();
                    let nodes = self.parse_alternation()?;
                    if self.is_eof() || self.current() != ')' {
                        return Err(ParseError::UnclosedGroup);
                    }
                    self.advance();
                    Ok(RegexNode::new_lookaround(
                        if negative {
                            LookaroundKind::NegativeLookahead
                        } else {
                            LookaroundKind::PositiveLookahead
                        },
                        nodes,
                    ))
                }
                _ => Err(ParseError::InvalidGroupSyntax),
            }
        } else {
            self.group_count += 1;
            let nodes = self.parse_alternation()?;
            if self.is_eof() || self.current() != ')' {
                return Err(ParseError::UnclosedGroup);
            }
            self.advance();
            Ok(RegexNode::new_group(GroupKind::Capturing(None), nodes))
        }
    }

    fn parse_group_name(&mut self) -> Result<String, ParseError> {
        let mut name = String::new();
        while !self.is_eof() && self.current() != '>' {
            if self.current().is_alphanumeric() || self.current() == '_' {
                name.push(self.current());
                self.advance();
            } else {
                return Err(ParseError::InvalidGroupName);
            }
        }

        if self.is_eof() || name.is_empty() {
            return Err(ParseError::InvalidGroupName);
        }

        self.advance(); // consume '>'
        Ok(name)
    }

    fn parse_escape(&mut self) -> Result<RegexNode, ParseError> {
        if self.is_eof() {
            return Err(ParseError::UnexpectedEndOfInput);
        }

        match self.current() {
            'b' => {
                self.advance();
                Ok(RegexNode::WordBoundary)
            }
            'k' => {
                self.advance();
                if !self.check_char('<') {
                    return Err(ParseError::InvalidBackreference);
                }
                self.advance();
                let name = self.parse_group_name()?;
                Ok(RegexNode::new_backreference(BackreferenceKind::NameBased(name)))
            }
            'w' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::Word))
            }
            'W' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::NotWord))
            }
            'd' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::Digit))
            }
            'D' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::NotDigit))
            }
            's' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::Whitespace))
            }
            'S' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::NotWhitespace))
            }
            'p' | 'P' => {
                let negated = self.current() == 'P';
                self.advance();
                self.parse_unicode_category(negated)
            }
            'n' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::NewLine,
                )))
            }
            't' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::Tab,
                )))
            }
            'r' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::CarriageReturn,
                )))
            }
            'f' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::FormFeed,
                )))
            }
            'v' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::VerticalTab,
                )))
            }
            '0' => {
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::Null,
                )))
            }
            'x' => {
                self.advance();
                let hex_value = self.parse_hex(2)?;
                Ok(RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::Hex(hex_value),
                )))
            }
            'u' => {
                self.advance();
                if !self.check_char('{') {
                    return Err(ParseError::InvalidUnicodeValue);
                }
                self.advance();
                let hex_value = self.parse_unicode_value()?;
                if !self.check_char('}') {
                    return Err(ParseError::InvalidUnicodeValue);
                }
                self.advance();
                Ok(RegexNode::new_character_type(CharacterTypeKind::EscapedChar(
                    EscapedChar::Unicode(hex_value),
                )))
            }
            c if c.is_ascii_digit() => {
                let num = self.parse_number()?;
                if num == 0 || num > self.group_count {
                    return Err(ParseError::InvalidBackreference);
                }
                Ok(RegexNode::new_backreference(BackreferenceKind::NumberBased(num)))
            }
            c => {
                self.advance();
                Ok(RegexNode::new_literal(c))
            }
        }
    }

    fn parse_unicode_category(&mut self, negated: bool) -> Result<RegexNode, ParseError> {
        if !self.check_char('{') {
            return Err(ParseError::InvalidUnicodeCategory);
        }
        self.advance();

        let category = match self.current() {
            'L' => UnicodeCategoryKind::Letter,
            'N' => UnicodeCategoryKind::Number,
            'P' => UnicodeCategoryKind::Punctuation,
            'S' => UnicodeCategoryKind::Symbol,
            'M' => UnicodeCategoryKind::Mark,
            'Z' => UnicodeCategoryKind::Separator,
            'C' => UnicodeCategoryKind::Other,
            _ => return Err(ParseError::InvalidUnicodeCategory),
        };
        self.advance();

        if !self.check_char('}') {
            return Err(ParseError::InvalidUnicodeCategory);
        }
        self.advance();

        Ok(RegexNode::new_unicode_category(category, negated))
    }

    fn parse_hex(&mut self, count: usize) -> Result<u32, ParseError> {
        let mut value = 0;
        for _ in 0..count {
            if self.is_eof() {
                return Err(ParseError::InvalidHexNumber);
            }
            let digit = self.current().to_digit(16)
                .ok_or(ParseError::InvalidHexNumber)?;
            value = value * 16 + digit;
            self.advance();
        }
        Ok(value)
    }

    fn parse_unicode_value(&mut self) -> Result<u32, ParseError> {
        let mut value = 0;
        let mut count = 0;
        while !self.is_eof() && self.current() != '}' && count < 6 {
            let digit = self.current().to_digit(16)
                .ok_or(ParseError::InvalidUnicodeValue)?;
            value = value * 16 + digit;
            self.advance();
            count += 1;
        }
        if count == 0 {
            return Err(ParseError::InvalidUnicodeValue);
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<usize, ParseError> {
        let mut num = 0;
        while !self.is_eof() && self.current().is_ascii_digit() {
            num = num * 10 + self.current().to_digit(10).unwrap() as usize;
            self.advance();
        }
        Ok(num)
    }

    fn check_str(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        let mut pos = self.position;
        
        for &c in &chars {
            if pos >= self.input.len() || self.input[pos] != c {
                return false;
            }
            pos += 1;
        }

        // If we matched the string, advance the position
        for _ in 0..chars.len() {
            self.advance();
        }
        true
    }

    fn check_char(&self, c: char) -> bool {
        !self.is_eof() && self.current() == c
    }

    fn current(&self) -> char {
        self.input[self.position]
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }
} 