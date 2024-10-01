use std::iter::Peekable;
use std::str::Chars;

use super::parser::Token;


#[derive(Debug)]
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        let chars = self.chars.by_ref();

        match chars.next() {
            Some(c) if c.is_whitespace() => Some(Token::Whitespace),
            Some(',') => Some(Token::Comma),
            Some('(') => Some(Token::LeftParenthesis),
            Some(')') => Some(Token::RightParenthesis),
            Some(c) if c.is_alphabetic() => {
                let mut identifier = String::new();
                identifier.push(c);
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() {
                        identifier.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if identifier.eq_ignore_ascii_case("SELECT") {
                    Some(Token::Keyword(identifier))
                } else {
                    Some(Token::Identifier(identifier))
                }
            }
            Some(c) if c.is_digit(10) => {
                let mut literal = String::new();
                literal.push(c);
                while let Some(&next) = chars.peek() {
                    if next.is_digit(10) {
                        literal.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                Some(Token::Literal(literal))
            }
            Some(c) => Some(Token::Operator(c)),
            None => Some(Token::Eof),
        }
    }
}
