use std::rc::Rc;

use crate::{
    RuntimeValue, Token,
    TokenType::{
        self, And, Bang, BangEqual, Class, EOF, Else, Equal, EqualEqual, False, For, Fun, Greater,
        GreaterEqual, Identifier, If, LeftParen, Less, LessEqual, Minus, Nil, Or, Plus, Print,
        Return, RightParen, Semicolon, Slash, Star, True, Var, While,
    },
    errors::{ParseError, XodyError},
    expr::Expr,
    stmt::Stmt,
};

#[allow(unused)]
pub struct AstPrinter;
#[allow(unused)]
impl AstPrinter {
    pub fn print(ex: Expr) {
        println!("{}", ex.to_string());
    }
}

pub struct Parser {
    tokens: Vec<Rc<Token>>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: &Vec<Rc<Token>>) -> Self {
        Self {
            tokens: tokens.clone(),
            current: 0,
        }
    }

    /// Checks if the current token is of toktype
    fn check(&self, toktype: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        return self.peek().toktype == *toktype;
    }

    fn advance(&mut self) -> Rc<Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        return self.previous();
    }

    /// Checks EOF
    fn is_at_end(&self) -> bool {
        self.peek().toktype == EOF
    }

    /// Peek the current token without consuming it
    fn peek(&self) -> Rc<Token> {
        self.tokens[self.current].clone()
    }

    /// Peek the previous token without consuming it
    fn previous(&self) -> Rc<Token> {
        self.tokens[self.current - 1].clone()
    }

    /// Returns true if the current token is one of toktypes.
    /// Consumes the token while checking it.
    fn matches(&mut self, toktypes: &[TokenType]) -> bool {
        for toktype in toktypes {
            if self.check(toktype) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expression(&mut self) -> Result<Rc<Expr>, ParseError> {
        self.assigment()
    }

    fn assigment(&mut self) -> Result<Rc<Expr>, ParseError> {
        let expr = self.or()?;

        if self.matches(&[Equal]) {
            let equals = self.previous().clone();
            let value = self.assigment()?;

            if let Expr::Variable { name } = (*expr).clone() {
                return Ok(Expr::new_assign(name, value));
            }

            return Err(ParseError::new(
                equals.clone(),
                "Invalid assignment target.",
            ));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Rc<Expr>, ParseError> {
        let mut expr = self.and()?;
        while self.matches(&[Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::new_logical(expr, operator, right);
        }

        return Ok(expr);
    }

    fn and(&mut self) -> Result<Rc<Expr>, ParseError> {
        let mut expr = self.equality()?;

        while self.matches(&[And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::new_logical(expr, operator, right)
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Rc<Expr>, ParseError> {
        let mut expr = self.comparison()?;
        while self.matches(&[BangEqual, EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Result::Ok(expr)
    }

    fn comparison(&mut self) -> Result<Rc<Expr>, ParseError> {
        let mut expr = self.term()?;
        while self.matches(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Result::Ok(expr)
    }

    fn term(&mut self) -> Result<Rc<Expr>, ParseError> {
        let mut expr = self.factor()?;
        while self.matches(&[Minus, Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Result::Ok(expr)
    }

    fn factor(&mut self) -> Result<Rc<Expr>, ParseError> {
        let mut expr = self.unary()?;
        while self.matches(&[Slash, Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Result::Ok(expr)
    }

    fn unary(&mut self) -> Result<Rc<Expr>, ParseError> {
        if self.matches(&[Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::new_unary(operator, right));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Rc<Expr>, ParseError> {
        if self.matches(&[False]) {
            return Ok(Expr::new_literal(RuntimeValue::new_bool(false)));
        }
        if self.matches(&[True]) {
            return Ok(Expr::new_literal(RuntimeValue::new_bool(true)));
        }
        if self.matches(&[Nil]) {
            return Ok(Expr::new_literal(RuntimeValue::new_nil()));
        }

        if self.matches(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::new_literal(Rc::new(
                self.previous()
                    .literal
                    .clone()
                    .expect("literal was not found"),
            )));
        }

        if self.matches(&[Identifier]) {
            return Ok(Expr::new_variable(self.previous()));
        }

        if self.matches(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(&RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::new_grouping(expr));
        }

        Err(ParseError::new(self.peek(), "Expect expression."))
    }

    /// Consumes the current token only if it's of type `toktype` returning it,
    /// else returns an error with the `message` string
    fn consume(&mut self, toktype: &TokenType, message: &str) -> Result<Rc<Token>, ParseError> {
        if self.check(toktype) {
            return Ok(self.advance());
        }

        Result::Err(ParseError::new(self.peek().clone(), message))
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().toktype == Semicolon {
                return;
            }

            match self.peek().toktype {
                Class | Fun | Var | For | If | While | Print | Return => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub fn parse(&mut self) -> Vec<Rc<Stmt>> {
        let mut statements: Vec<Rc<Stmt>> = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(val) => statements.push(val),
                Err(err) => err.throw(),
            }
        }
        statements
    }

    fn declaration(&mut self) -> Result<Rc<Stmt>, ParseError> {
        if self.matches(&[TokenType::Var]) {
            match self.var_declaration() {
                Ok(val) => return Result::Ok(val),
                Err(_err) => self.synchronize(),
            }
        }

        self.statement()
    }

    fn statement(&mut self) -> Result<Rc<Stmt>, ParseError> {
        if self.matches(&[TokenType::If]) {
            return Ok(self.if_statement()?);
        }

        if self.matches(&[TokenType::Print]) {
            return Ok(self.print_statement()?);
        }

        if self.matches(&[TokenType::While]) {
            return Ok(self.while_statement()?);
        }

        if self.matches(&[TokenType::LeftBrace]) {
            return Ok(Rc::new(Stmt::Block {
                statements: self.block()?,
            }));
        }

        Result::Ok(self.expression_statement()?)
    }

    fn while_statement(&mut self) -> Result<Rc<Stmt>, ParseError> {
        self.consume(&LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expect ')' after if condition.")?;
        let body = self.statement()?;
        Ok(Rc::new(Stmt::While { condition, body }))
    }

    fn if_statement(&mut self) -> Result<Rc<Stmt>, ParseError> {
        self.consume(&LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;

        let else_branch = if self.matches(&[Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(Rc::new(Stmt::If {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn block(&mut self) -> Result<Vec<Rc<Stmt>>, ParseError> {
        let mut statements: Vec<Rc<Stmt>> = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(&TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Rc<Stmt>, ParseError> {
        let expr = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after value.")?;

        Ok(Rc::new(Stmt::Print { expr }))
    }

    fn expression_statement(&mut self) -> Result<Rc<Stmt>, ParseError> {
        let expr = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after value.")?;

        Ok(Rc::new(Stmt::Expression { expr }))
    }

    fn var_declaration(&mut self) -> Result<Rc<Stmt>, ParseError> {
        let name = self
            .consume(&TokenType::Identifier, "Expect variable name.")?
            .clone();

        let mut initializer = Expr::new_literal(RuntimeValue::new_nil());

        if self.matches(&[TokenType::Equal]) {
            initializer = self.expression()?;
        }

        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Rc::new(Stmt::Var { name, initializer }))
    }
}
