use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(f64),
    String(String),
    Equals,
    Comma,
    Dot,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Keyword(String),
    Operator(String),
    Eof,
}

pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
    pub tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
            tokens: Vec::new(),
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.source.peek().cloned()
    }

    fn advance(&mut self) -> Option<char> {
        self.source.next()
    }

    fn consume_while<F>(&mut self, mut predicate: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut result = String::new();
        while let Some(&c) = self.source.peek() {
            if predicate(c) {
                result.push(c);
                self.source.next();
            } else {
                break;
            }
        }
        result
    }

    pub fn tokenize(mut self) -> Vec<Token> {
        while let Some(c) = self.advance() {
            match c {
                ' ' | '\t' | '\r' | '\n' => continue,
                '=' => self.tokens.push(Token::Equals),
                ',' => self.tokens.push(Token::Comma),
                '.' => self.tokens.push(Token::Dot),
                ';' => self.tokens.push(Token::Semicolon),
                '(' => self.tokens.push(Token::LParen),
                ')' => self.tokens.push(Token::RParen),
                '{' => self.tokens.push(Token::LBrace),
                '}' => self.tokens.push(Token::RBrace),
                '"' => {
                    let literal = self.consume_while(|ch| ch != '"');
                    self.advance();
                    self.tokens.push(Token::String(literal));
                }
                '#' => {
                    if self.peek() == Some('"') {
                        self.advance();
                        while let Some(ch) = self.advance() {
                            if ch == '"' {
                                break;
                            }
                        }
                    } else {
                        self.consume_while(|ch| ch != '\n');
                    }
                }
                c if c.is_ascii_digit() => {
                    let mut literal = c.to_string();
                    literal.push_str(&self.consume_while(|ch| ch.is_ascii_digit() || ch == '.'));
                    let value = literal.parse::<f64>().unwrap_or(0.0);
                    self.tokens.push(Token::Number(value));
                }
                c if is_identifier_start(c) => {
                    let mut identifier = c.to_string();
                    identifier.push_str(&self.consume_while(is_identifier_part));
                    let token = match identifier.as_str() {
                        "func" | "return" | "if" | "else" | "while" | "local" | "NAME" | "DESC" => {
                            Token::Keyword(identifier)
                        }
                        _ => Token::Identifier(identifier),
                    };
                    self.tokens.push(token);
                }
                '+' | '-' | '*' | '/' | '<' | '>' | '!' => {
                    let operator = if self.peek() == Some('=') {
                        let first = c;
                        self.advance();
                        format!("{}=", first)
                    } else {
                        c.to_string()
                    };
                    self.tokens.push(Token::Operator(operator));
                }
                _ => {
                    self.tokens.push(Token::Operator(c.to_string()));
                }
            }
        }
        self.tokens.push(Token::Eof);
        self.tokens
    }
}

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_identifier_part(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
