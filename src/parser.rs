use crate::ast::{AnchorType, BackreferenceKind, GroupKind, Quantifier, RegexNode};

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
        let mut nodes = Vec::new();
        while !self.is_eof() {
            nodes.push(self.parse_node()?);
        }
        Ok(nodes)
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
        
        let kind = if self.check_str("?:") {
            GroupKind::NonCapturing
        } else if self.check_char('?') {
            self.advance();
            if self.check_char('<') {
                self.advance();
                let name = self.parse_group_name()?;
                GroupKind::Capturing(Some(name))
            } else {
                return Err(ParseError::InvalidGroupSyntax);
            }
        } else {
            self.group_count += 1;
            GroupKind::Capturing(None)
        };

        let mut nodes = Vec::new();
        while !self.is_eof() && self.current() != ')' {
            nodes.push(self.parse_node()?);
        }

        if self.is_eof() {
            return Err(ParseError::UnclosedGroup);
        }

        self.advance(); // consume ')'
        Ok(RegexNode::new_group(kind, nodes))
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