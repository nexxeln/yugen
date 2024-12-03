use crate::ast::{AnchorType, RegexNode};

pub struct Parser {
    input: Vec<char>,
    position: usize,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEndOfInput,
    UnexpectedCharacter(char),
    UnclosedCharacterClass,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Parser {
            input: input.chars().collect(),
            position: 0,
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

        match self.current() {
            '.' => {
                self.advance();
                Ok(RegexNode::Dot)
            }
            '^' => {
                self.advance();
                Ok(RegexNode::new_anchor(AnchorType::Start))
            }
            '$' => {
                self.advance();
                Ok(RegexNode::new_anchor(AnchorType::End))
            }
            '\\' => {
                self.advance();
                self.parse_escape()
            }
            '[' => self.parse_character_class(),
            c => {
                self.advance();
                Ok(RegexNode::new_literal(c))
            }
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

    fn parse_escape(&mut self) -> Result<RegexNode, ParseError> {
        if self.is_eof() {
            return Err(ParseError::UnexpectedEndOfInput);
        }

        match self.current() {
            'b' => {
                self.advance();
                Ok(RegexNode::WordBoundary)
            }
            c => {
                self.advance();
                Ok(RegexNode::new_literal(c))
            }
        }
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