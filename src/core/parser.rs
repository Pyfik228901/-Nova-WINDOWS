use super::ast::*;
use super::token::{Token, TokenType};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ParserError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Syntax error at line {}, col {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ParserError {}

pub struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            // Skip semicolons
            if self.match_token(&[TokenType::Semicolon]) {
                continue;
            }
            statements.push(self.statement()?);
        }
        Ok(Program { statements })
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_token(&[TokenType::Let]) {
            self.let_statement()
        } else if self.match_token(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_token(&[TokenType::For]) {
            self.for_statement()
        } else if self.check(&TokenType::Func) && self.peek_next_is_ident() {
            // Function definition statement: func name(...) { ... }
            self.advance();
            self.func_statement()
        } else if self.match_token(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_token(&[TokenType::Import]) {
            self.import_statement()
        } else if self.match_token(&[TokenType::Extern]) {
            self.extern_statement()
        } else if self.match_token(&[TokenType::LBrace]) {
            let stmts = self.block()?;
            Ok(Stmt::Block(stmts))
        } else {
            self.expr_statement()
        }
    }

    fn peek_next_is_ident(&self) -> bool {
        if self.pos + 1 < self.tokens.len() {
            matches!(self.tokens[self.pos + 1].token_type, TokenType::Ident(_))
        } else {
            false
        }
    }

    fn let_statement(&mut self) -> Result<Stmt, ParserError> {
        let name = match self.peek().token_type.clone() {
            TokenType::Ident(name) => {
                self.advance();
                name
            }
            _ => {
                return Err(self.error("Expected variable name after 'let'"));
            }
        };

        self.consume(&TokenType::Assign, "Expected '=' after variable name")?;
        let init = self.expr()?;
        self.match_token(&[TokenType::Semicolon]);
        Ok(Stmt::VarDecl { name, init })
    }

    fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(&TokenType::LParen, "Expected '(' after 'print'")?;
        let expr = self.expr()?;
        self.consume(&TokenType::RParen, "Expected ')' after print argument")?;
        self.match_token(&[TokenType::Semicolon]);
        Ok(Stmt::Print(expr))
    }

    fn if_statement(&mut self) -> Result<Stmt, ParserError> {
        let condition = self.expr()?;
        self.consume(&TokenType::LBrace, "Expected '{' after if condition")?;
        let then_branch = self.block()?;
        let else_branch = if self.match_token(&[TokenType::Else]) {
            if self.match_token(&[TokenType::If]) {
                Some(vec![self.if_statement()?])
            } else {
                self.consume(&TokenType::LBrace, "Expected '{' after 'else'")?;
                Some(self.block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParserError> {
        let condition = self.expr()?;
        self.consume(&TokenType::LBrace, "Expected '{' after while condition")?;
        let body = self.block()?;
        Ok(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Stmt, ParserError> {
        let item = match self.peek().token_type.clone() {
            TokenType::Ident(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("Expected loop variable name after 'for'")),
        };

        self.consume(&TokenType::In, "Expected 'in' after for variable")?;
        let iterable = self.expr()?;
        self.consume(&TokenType::LBrace, "Expected '{' after for iterable")?;
        let body = self.block()?;
        Ok(Stmt::For { item, iterable, body })
    }

    fn func_statement(&mut self) -> Result<Stmt, ParserError> {
        let name = match self.peek().token_type.clone() {
            TokenType::Ident(n) => {
                self.advance();
                n
            }
            _ => return Err(self.error("Expected function name")),
        };

        self.consume(&TokenType::LParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(&TokenType::RParen) {
            loop {
                match self.peek().token_type.clone() {
                    TokenType::Ident(p) => {
                        self.advance();
                        params.push(p);
                    }
                    _ => return Err(self.error("Expected parameter name")),
                }
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(&TokenType::RParen, "Expected ')' after parameters")?;
        self.consume(&TokenType::LBrace, "Expected '{' before function body")?;
        let body = self.block()?;
        Ok(Stmt::FuncDef { name, params, body })
    }

    fn return_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = if self.check(&TokenType::Semicolon)
            || self.check(&TokenType::RBrace)
            || self.is_at_end()
        {
            None
        } else {
            Some(self.expr()?)
        };
        self.match_token(&[TokenType::Semicolon]);
        Ok(Stmt::Return(expr))
    }

    fn import_statement(&mut self) -> Result<Stmt, ParserError> {
        let is_grass = if let TokenType::Ident(id) = &self.peek().token_type {
            if id == "grass" || id == "gg" {
                self.advance();
                true
            } else {
                false
            }
        } else {
            false
        };

        let module_path = match self.peek().token_type.clone() {
            TokenType::String(s) => {
                self.advance();
                s
            }
            _ => return Err(self.error("Expected string module path after 'import'")),
        };

        let mut alias = None;
        if self.match_token(&[TokenType::As]) {
            match self.peek().token_type.clone() {
                TokenType::Ident(id) => {
                    self.advance();
                    alias = Some(id);
                }
                _ => return Err(self.error("Expected alias identifier after 'as'")),
            }
        }

        self.match_token(&[TokenType::Semicolon]);
        Ok(Stmt::Import {
            module_path,
            alias,
            is_grass,
        })
    }

    fn extern_statement(&mut self) -> Result<Stmt, ParserError> {
        let language = match self.peek().token_type.clone() {
            TokenType::String(s) => {
                self.advance();
                s
            }
            _ => return Err(self.error("Expected language name string after 'extern', e.g. extern \"grass\"")),
        };

        self.consume(&TokenType::LBrace, "Expected '{' after extern language")?;
        let start_offset = self.previous().span.1;

        let mut depth = 1;
        while depth > 0 && !self.is_at_end() {
            if self.peek().token_type == TokenType::LBrace {
                depth += 1;
            } else if self.peek().token_type == TokenType::RBrace {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            self.advance();
        }

        let end_offset = self.peek().span.0;
        let raw_code = if start_offset <= end_offset && end_offset <= self.source.len() {
            self.source[start_offset..end_offset].trim().to_string()
        } else {
            String::new()
        };

        self.consume(&TokenType::RBrace, "Expected '}' after extern block")?;
        self.match_token(&[TokenType::Semicolon]);

        Ok(Stmt::ExternBlock {
            language,
            code: raw_code,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            if self.match_token(&[TokenType::Semicolon]) {
                continue;
            }
            statements.push(self.statement()?);
        }
        self.consume(&TokenType::RBrace, "Expected '}' after block")?;
        Ok(statements)
    }

    fn expr_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expr()?;

        // Check for compound assignment or simple assignment
        if self.match_token(&[
            TokenType::Assign,
            TokenType::PlusAssign,
            TokenType::MinusAssign,
            TokenType::MulAssign,
            TokenType::DivAssign,
        ]) {
            let op_token = self.previous().token_type.clone();
            let value = self.expr()?;
            self.match_token(&[TokenType::Semicolon]);

            let target = match expr {
                Expr::Variable(name) => AssignTarget::Variable(name),
                Expr::MemberAccess { object, member } => AssignTarget::Member(object, member),
                Expr::IndexAccess { object, index } => AssignTarget::Index(object, index),
                _ => return Err(self.error("Invalid assignment target")),
            };

            let op = match op_token {
                TokenType::Assign => None,
                TokenType::PlusAssign => Some(BinaryOp::Add),
                TokenType::MinusAssign => Some(BinaryOp::Sub),
                TokenType::MulAssign => Some(BinaryOp::Mul),
                TokenType::DivAssign => Some(BinaryOp::Div),
                _ => None,
            };

            return Ok(Stmt::Assignment { target, op, value });
        }

        self.match_token(&[TokenType::Semicolon]);
        Ok(Stmt::Expr(expr))
    }

    pub fn expr(&mut self) -> Result<Expr, ParserError> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.logical_and()?;
        while self.match_token(&[TokenType::Or]) {
            let right = self.logical_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.equality()?;
        while self.match_token(&[TokenType::And]) {
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;
        while self.match_token(&[TokenType::Eq, TokenType::NotEq]) {
            let op = match self.previous().token_type {
                TokenType::Eq => BinaryOp::Eq,
                TokenType::NotEq => BinaryOp::NotEq,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term()?;
        while self.match_token(&[TokenType::Lt, TokenType::Gt, TokenType::LtEq, TokenType::GtEq]) {
            let op = match self.previous().token_type {
                TokenType::Lt => BinaryOp::Lt,
                TokenType::Gt => BinaryOp::Gt,
                TokenType::LtEq => BinaryOp::LtEq,
                TokenType::GtEq => BinaryOp::GtEq,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;
        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let op = match self.previous().token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;
        while self.match_token(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let op = match self.previous().token_type {
                TokenType::Star => BinaryOp::Mul,
                TokenType::Slash => BinaryOp::Div,
                TokenType::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.match_token(&[TokenType::Not, TokenType::Minus]) {
            let op = match self.previous().token_type {
                TokenType::Not => UnaryOp::Not,
                TokenType::Minus => UnaryOp::Neg,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(right),
            });
        }
        self.call_and_access()
    }

    fn call_and_access(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LParen]) {
                let mut args = Vec::new();
                if !self.check(&TokenType::RParen) {
                    loop {
                        args.push(self.expr()?);
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(&TokenType::RParen, "Expected ')' after function arguments")?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else if self.match_token(&[TokenType::Dot]) {
                let member = match self.peek().token_type.clone() {
                    TokenType::Ident(name) => {
                        self.advance();
                        name
                    }
                    _ => return Err(self.error("Expected property or method name after '.'")),
                };
                expr = Expr::MemberAccess {
                    object: Box::new(expr),
                    member,
                };
            } else if self.match_token(&[TokenType::LBracket]) {
                let index = self.expr()?;
                self.consume(&TokenType::RBracket, "Expected ']' after index")?;
                expr = Expr::IndexAccess {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        let tok = self.peek().clone();
        match tok.token_type {
            TokenType::Int(i) => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Int(i)))
            }
            TokenType::Float(f) => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Float(f)))
            }
            TokenType::String(s) => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::String(s)))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Bool(true)))
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Bool(false)))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Null))
            }
            TokenType::Ident(name) => {
                self.advance();
                Ok(Expr::Variable(name))
            }
            TokenType::LParen => {
                self.advance();
                let expr = self.expr()?;
                self.consume(&TokenType::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            TokenType::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&TokenType::RBracket) {
                    loop {
                        elements.push(self.expr()?);
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(&TokenType::RBracket, "Expected ']' after array elements")?;
                Ok(Expr::ArrayLiteral(elements))
            }
            TokenType::LBrace => {
                // Map/Object literal: { key: value, ... }
                self.advance();
                let mut pairs = Vec::new();
                if !self.check(&TokenType::RBrace) {
                    loop {
                        let key = match self.peek().token_type.clone() {
                            TokenType::Ident(k) => {
                                self.advance();
                                k
                            }
                            TokenType::String(k) => {
                                self.advance();
                                k
                            }
                            _ => return Err(self.error("Expected string or identifier key in map literal")),
                        };
                        self.consume(&TokenType::Colon, "Expected ':' after map key")?;
                        let val = self.expr()?;
                        pairs.push((key, val));
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(&TokenType::RBrace, "Expected '}' after map literal")?;
                Ok(Expr::MapLiteral(pairs))
            }
            TokenType::Func => {
                // Anonymous or lambda function expression: func(params) { body }
                self.advance();
                let name = if let TokenType::Ident(n) = &self.peek().token_type {
                    let n = n.clone();
                    self.advance();
                    Some(n)
                } else {
                    None
                };
                self.consume(&TokenType::LParen, "Expected '(' after 'func'")?;
                let mut params = Vec::new();
                if !self.check(&TokenType::RParen) {
                    loop {
                        match self.peek().token_type.clone() {
                            TokenType::Ident(p) => {
                                self.advance();
                                params.push(p);
                            }
                            _ => return Err(self.error("Expected parameter name")),
                        }
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(&TokenType::RParen, "Expected ')' after parameters")?;
                self.consume(&TokenType::LBrace, "Expected '{' before function body")?;
                let body = self.block()?;
                Ok(Expr::FunctionExpr { name, params, body })
            }
            _ => Err(self.error(&format!("Unexpected token '{:?}'", tok.token_type))),
        }
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, ParserError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    fn error(&self, message: &str) -> ParserError {
        let tok = self.peek();
        ParserError {
            message: message.to_string(),
            line: tok.line,
            column: tok.column,
        }
    }
}
