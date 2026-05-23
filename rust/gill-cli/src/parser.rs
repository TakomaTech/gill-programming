use crate::ast::{Expr, Literal, Program, Statement};
use crate::error::GillError;
use crate::lexer::{Lexer, Token};

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let tokens = Lexer::new(source).tokenize();
        Self { tokens, current: 0, source }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !matches!(self.peek(), Token::Eof) {
            self.current += 1;
        }
        self.previous()
    }

    fn match_token(&mut self, expected: &[Token]) -> bool {
        for token in expected {
            if self.peek() == token {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, expected: Token, message: &str) -> Result<&Token, GillError> {
        if self.peek() == &expected {
            Ok(self.advance())
        } else {
            Err(GillError::UnexpectedToken(message.to_string()))
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, GillError> {
        let mut metadata = Vec::new();
        let mut body = Vec::new();

        while !matches!(self.peek(), Token::Eof) {
            if let Token::Keyword(ref key) = self.peek() {
                if key == "NAME" || key == "DESC" {
                    self.advance();
                    self.consume(Token::Equals, "Expected '=' after metadata key")?;
                    if let Token::String(value) = self.advance().clone() {
                        metadata.push((key.clone(), value));
                        continue;
                    }
                    return Err(GillError::UnexpectedToken("Expected string after metadata key".to_string()));
                }
            }

            if self.peek() == &Token::Eof {
                break;
            }

            body.push(self.parse_statement()?);
        }

        Ok(Program { metadata, body })
    }

    fn parse_statement(&mut self) -> Result<Statement, GillError> {
        match self.peek() {
            Token::Keyword(keyword) if keyword == "func" => self.parse_function_decl(),
            Token::Keyword(keyword) if keyword == "return" => self.parse_return(),
            Token::Keyword(keyword) if keyword == "if" => self.parse_if(),
            Token::Keyword(keyword) if keyword == "while" => self.parse_while(),
            Token::Keyword(keyword) if keyword == "local" => self.parse_variable_declaration(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_function_decl(&mut self) -> Result<Statement, GillError> {
        self.advance();
        let name = if let Token::Identifier(name) = self.advance().clone() {
            name
        } else {
            return Err(GillError::UnexpectedToken("Expected function name".to_string()));
        };
        self.consume(Token::LParen, "Expected '(' after function name")?;
        let mut parameters = Vec::new();
        if !matches!(self.peek(), Token::RParen) {
            loop {
                if let Token::Identifier(param) = self.advance().clone() {
                    parameters.push(param);
                } else {
                    return Err(GillError::UnexpectedToken("Expected function parameter".to_string()));
                }
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }
        self.consume(Token::RParen, "Expected ')' after parameters")?;
        self.consume(Token::LBrace, "Expected '{' for function body")?;
        let body = self.parse_block()?;
        Ok(Statement::FunctionDeclaration { name, parameters, body })
    }

    fn parse_return(&mut self) -> Result<Statement, GillError> {
        self.advance();
        if self.peek() == &Token::Semicolon {
            self.advance();
            return Ok(Statement::Return(None));
        }
        let expr = self.parse_expression()?;
        self.consume(Token::Semicolon, "Expected ';' after return expression")?;
        Ok(Statement::Return(Some(expr)))
    }

    fn parse_if(&mut self) -> Result<Statement, GillError> {
        self.advance();
        self.consume(Token::LParen, "Expected '(' after if")?;
        let condition = self.parse_expression()?;
        self.consume(Token::RParen, "Expected ')' after condition")?;
        self.consume(Token::LBrace, "Expected '{' after if condition")?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.peek() == &Token::Keyword("else".into()) {
            self.advance();
            self.consume(Token::LBrace, "Expected '{' after else")?;
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Statement::If { condition, then_branch, else_branch })
    }

    fn parse_while(&mut self) -> Result<Statement, GillError> {
        self.advance();
        self.consume(Token::LParen, "Expected '(' after while")?;
        let condition = self.parse_expression()?;
        self.consume(Token::RParen, "Expected ')' after while condition")?;
        self.consume(Token::LBrace, "Expected '{' after while condition")?;
        let body = self.parse_block()?;
        Ok(Statement::While { condition, body })
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, GillError> {
        self.advance();
        let name = if let Token::Identifier(name) = self.advance().clone() {
            name
        } else {
            return Err(GillError::UnexpectedToken("Expected variable name after local".to_string()));
        };
        self.consume(Token::Equals, "Expected '=' after variable name")?;
        let value = self.parse_expression()?;
        self.consume(Token::Semicolon, "Expected ';' after variable declaration")?;
        Ok(Statement::VariableDeclaration { name, value })
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, GillError> {
        let expr = self.parse_expression()?;
        self.consume(Token::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::Expression(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, GillError> {
        let mut body = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            body.push(self.parse_statement()?);
        }
        self.consume(Token::RBrace, "Expected '}' after block")?;
        Ok(body)
    }

    fn parse_expression(&mut self) -> Result<Expr, GillError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, GillError> {
        let expr = self.parse_equality()?;
        if self.peek() == &Token::Equals {
            self.advance();
            let value = self.parse_assignment()?;
            if let Expr::Identifier(name) = expr {
                return Ok(Expr::Binary { left: Box::new(Expr::Identifier(name)), operator: "=".to_string(), right: Box::new(value) });
            }
            return Err(GillError::InvalidExpression);
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, GillError> {
        let mut expr = self.parse_comparison()?;
        while let Token::Operator(op) = self.peek().clone() {
            if op == "==" || op == "!=" {
                self.advance();
                let right = self.parse_comparison()?;
                expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, GillError> {
        let mut expr = self.parse_term()?;
        while let Token::Operator(op) = self.peek().clone() {
            if op == ">" || op == ">=" || op == "<" || op == "<=" {
                self.advance();
                let right = self.parse_term()?;
                expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, GillError> {
        let mut expr = self.parse_factor()?;
        while let Token::Operator(op) = self.peek().clone() {
            if op == "+" || op == "-" {
                self.advance();
                let right = self.parse_factor()?;
                expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, GillError> {
        let mut expr = self.parse_unary()?;
        while let Token::Operator(op) = self.peek().clone() {
            if op == "*" || op == "/" {
                self.advance();
                let right = self.parse_unary()?;
                expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, GillError> {
        if let Token::Operator(op) = self.peek().clone() {
            if op == "!" || op == "-" {
                self.advance();
                let right = self.parse_unary()?;
                return Ok(Expr::Unary { operator: op, right: Box::new(right) });
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, GillError> {
        match self.advance().clone() {
            Token::Number(value) => Ok(Expr::Literal(Literal::Number(value))),
            Token::String(value) => Ok(Expr::Literal(Literal::String(value))),
            Token::Identifier(name) => self.finish_identifier(name),
            Token::Keyword(keyword) if keyword == "true" => Ok(Expr::Literal(Literal::Boolean(true))),
            Token::Keyword(keyword) if keyword == "false" => Ok(Expr::Literal(Literal::Boolean(false))),
            Token::LParen => {
                let expr = self.parse_expression()?;
                self.consume(Token::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            token => Err(GillError::UnexpectedToken(format!("Unexpected primary token: {:?}", token))),
        }
    }

    fn finish_identifier(&mut self, name: String) -> Result<Expr, GillError> {
        let mut expr = Expr::Identifier(name);
        loop {
            if self.peek() == &Token::Dot {
                self.advance();
                if let Token::Identifier(property) = self.advance().clone() {
                    expr = Expr::Access { object: Box::new(expr), property };
                    continue;
                }
                return Err(GillError::UnexpectedToken("Expected property name after '.'".to_string()));
            }
            if self.peek() == &Token::LParen {
                self.advance();
                let mut arguments = Vec::new();
                if !matches!(self.peek(), Token::RParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if !self.match_token(&[Token::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(Token::RParen, "Expected ')' after call arguments")?;
                expr = Expr::Call { callee: Box::new(expr), arguments };
                continue;
            }
            break;
        }
        Ok(expr)
    }
}
