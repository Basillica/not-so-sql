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


pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                // Skip whitespace
                ' ' | '\t' | '\n' => {
                    chars.next(); // Consume the character
                    tokens.push(Token::Whitespace);
                }
                // Handle keywords and identifiers
                'A'..='Z' | 'a'..='z' => {
                    let mut word = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() {
                            word.push(c);
                            chars.next(); // Consume the character
                        } else {
                            break;
                        }
                    }
                    // Check if it's a keyword
                    let token = match word.to_uppercase().as_str() {
                        "SELECT" | "FROM" | "WHERE" => Token::Keyword(word),
                        _ => Token::Identifier(word),
                    };
                    tokens.push(token);
                }
                // Handle numeric literals
                '0'..='9' => {
                    let mut number = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_numeric() {
                            number.push(c);
                            chars.next(); // Consume the character
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Literal(number));
                }
                // Handle symbols
                ',' => {
                    chars.next();
                    tokens.push(Token::Comma);
                }
                '(' => {
                    chars.next();
                    tokens.push(Token::LeftParenthesis);
                }
                ')' => {
                    chars.next();
                    tokens.push(Token::RightParenthesis);
                }
                _ => {
                    // Handle unexpected characters
                    chars.next();
                    tokens.push(Token::Operator(ch));
                }
            }
        }

        tokens.push(Token::Eof); // End of file token
        tokens
    }
}