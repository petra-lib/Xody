use crate::{
    RuntimeType, Token,
    TokenType::{
        self, Bang, BangEqual, Class, EOF, Equal, EqualEqual, False, For, Fun, Greater,
        GreaterEqual, Identifier, If, LeftParen, Less, LessEqual, Minus, Nil, Plus, Print, Return,
        RightParen, Semicolon, Slash, Star, True, Var, While,
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
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: &Vec<Token>) -> Self {
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

    fn advance(&mut self) -> &Token {
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
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Peek the previous token without consuming it
    fn previous<'a, 'b>(&'b self) -> &'a Token
    where
        'b: 'a,
    {
        &self.tokens[self.current - 1]
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

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assigment()
    }

    fn assigment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;

        if self.matches(&[Equal]) {
            let equals = self.previous().clone();
            let value = self.assigment()?;

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }

            return Err(ParseError::new(
                equals.clone(),
                "Invalid assignment target.",
            ));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.matches(&[BangEqual, EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Result::Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.matches(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Result::Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.matches(&[Minus, Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Result::Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.matches(&[Slash, Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Result::Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[False]) {
            return Ok(Expr::Literal {
                value: RuntimeType::Bool(false),
            });
        }
        if self.matches(&[True]) {
            return Ok(Expr::Literal {
                value: RuntimeType::Bool(true),
            });
        }
        if self.matches(&[Nil]) {
            return Ok(Expr::Literal {
                value: RuntimeType::Nil,
            });
        }

        if self.matches(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal {
                value: self
                    .previous()
                    .literal
                    .clone()
                    .expect("Literal was not found"),
            });
        }

        if self.matches(&[Identifier]) {
            return Ok(Expr::Variable {
                name: self.previous().clone(),
            });
        }

        if self.matches(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(&RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        Err(ParseError::new(self.peek().clone(), "Expect expression."))
    }

    /// Consumes the current token only if it's of type `toktype` returning it,
    /// else returns an error with the `message` string
    fn consume(&mut self, toktype: &TokenType, message: &str) -> Result<&Token, ParseError> {
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

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(val) => statements.push(val),
                Err(err) => err.throw(),
            }
        }
        statements
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.matches(&[TokenType::Var]) {
            match self.var_declaration() {
                Ok(val) => return Result::Ok(val),
                Err(err) => self.synchronize(),
            }
        }

        self.statement()
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.matches(&[TokenType::Print]) {
            return Result::Ok(self.print_statement()?);
        }

        if self.matches(&[TokenType::LeftBrace]) {
            return Result::Ok(Stmt::Block {
                statements: self.block()?,
            });
        }

        Result::Ok(self.expression_statement()?)
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(&TokenType::RightBrace, "Expect '}' after block.");
        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after value.")?;

        Result::Ok(Stmt::Print {
            expr: Box::new(expr),
        })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after value.")?;

        Result::Ok(Stmt::Expression {
            expr: Box::new(expr),
        })
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(&TokenType::Identifier, "Expect variable name.")?
            .clone();

        let mut initializer = Expr::Literal {
            value: RuntimeType::Nil,
        };

        if self.matches(&[TokenType::Equal]) {
            initializer = self.expression()?;
        }

        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Result::Ok(Stmt::Var {
            name: name.clone(),
            initializer: Box::new(initializer),
        })
    }
}
