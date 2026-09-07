use crate::{
    RuntimeType, Token,
    TokenType::{
        self, Bang, BangEqual, Class, EOF, EqualEqual, False, For, Fun, Greater, GreaterEqual, If,
        LeftParen, Less, LessEqual, Minus, Nil, Plus, Print, Return, RightParen, Semicolon, Slash,
        Star, True, Var, While,
    },
    errors::ParseError,
    expr::Expr,
};

macro_rules! parenthesize {
    ($name: expr, $($exprs: expr),*) => {{
        let mut output = String::new();
        output += "(";
        output += &$name;
        $(
            output += " ";
            output += &AstPrinter::stringify(*$exprs);
        )*
        output += ")";
        output
    }};
}

#[allow(unused)]
pub struct AstPrinter;
#[allow(unused)]
impl AstPrinter {
    pub fn print(ex: Expr) {
        println!("{}", Self::stringify(ex));
    }

    pub fn stringify(ex: Expr) -> String {
        match ex {
            Expr::Binary {
                left,
                operator,
                right,
            } => parenthesize!(operator.lexeme, left, right),
            Expr::Grouping { expression } => parenthesize!("group", expression),
            Expr::Unary { operator, right } => parenthesize!(operator.lexeme, right),
            Expr::Literal { value } => match value {
                RuntimeType::Number(val) => val.to_string(),
                RuntimeType::String(val) => val.to_string(),
                RuntimeType::Bool(val) => val.to_string(),
                RuntimeType::Nil => "nil".to_string(),
            },
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[allow(unused)]
impl Parser {
    pub fn new(tokens: &Vec<Token>) -> Self {
        Self {
            tokens: tokens.clone(),
            current: 0,
        }
    }

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

    fn is_at_end(&self) -> bool {
        self.peek().toktype == EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous<'a, 'b>(&'b self) -> &'a Token
    where
        'b: 'a,
    {
        &self.tokens[self.current - 1]
    }

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
        self.equality()
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

        if self.matches(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(&RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        Err(ParseError::new(self.peek().clone(), "Expect expression."))
    }

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

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }
}
