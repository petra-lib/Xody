use crate::{
    RuntimeValue, Token,
    TokenType::{
        self, And, Bang, BangEqual, Class, EOF, Else, Equal, EqualEqual, False, For, Fun, Greater,
        GreaterEqual, Identifier, If, LeftParen, Less, LessEqual, Minus, Nil, Or, Plus, Print,
        Return, RightParen, Semicolon, Slash, Star, True, Var, While,
    },
    ast_arena::{AstArena, ExprIdx, StmtIdx, TokenIdx},
    errors::ParseError,
    expr::Expr,
    stmt::Stmt,
};

pub struct Parser<'a> {
    current_token: usize,
    ast_arena: &'a mut AstArena,
}

impl<'a> Parser<'a> {
    pub fn new(ast_arena: &'a mut AstArena) -> Self {
        Self {
            current_token: 0,
            ast_arena,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<StmtIdx>, ParseError> {
        self.ast_arena.clear_stmt();
        self.ast_arena.clear_expr();
        let mut statements: Vec<StmtIdx> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    /// Checks if the current token is of toktype
    fn check(&self, toktype: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        return self.peek().toktype == *toktype;
    }

    fn advance(&mut self) -> TokenIdx {
        if !self.is_at_end() {
            self.current_token += 1;
        }
        return self.previous();
    }

    /// Checks EOF
    fn is_at_end(&self) -> bool {
        self.peek().toktype == EOF
    }

    /// Peek the current token without consuming it
    fn peek(&self) -> &Token {
        self.ast_arena.get_token(TokenIdx(self.current_token))
    }

    /// Peek the previous token without consuming it
    fn previous(&self) -> TokenIdx {
        TokenIdx(self.current_token - 1)
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

    fn expression(&mut self) -> Result<ExprIdx, ParseError> {
        self.assigment()
    }

    fn assigment(&mut self) -> Result<ExprIdx, ParseError> {
        let expr_id = self.or()?;

        if self.matches(&[Equal]) {
            let equals = self.previous();
            let value = self.assigment()?;

            if let Expr::Variable { name } = self.ast_arena.get_expr(expr_id) {
                return Ok(self.ast_arena.insert_expr(Expr::Assign {
                    name: name.clone(),
                    value,
                }));
            }

            return Err(ParseError::new(
                self.ast_arena.get_token(equals).clone(),
                "Invalid assignment target.",
            ));
        }

        Ok(expr_id)
    }

    fn or(&mut self) -> Result<ExprIdx, ParseError> {
        let mut expr_id = self.and()?;
        while self.matches(&[Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr_id = self.ast_arena.insert_expr(Expr::Logical {
                left: expr_id,
                operator,
                right,
            })
        }

        return Ok(expr_id);
    }

    fn and(&mut self) -> Result<ExprIdx, ParseError> {
        let mut expr_id = self.equality()?;

        while self.matches(&[And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr_id = self.ast_arena.insert_expr(Expr::Logical {
                left: expr_id,
                operator,
                right,
            });
        }

        Ok(expr_id)
    }

    fn equality(&mut self) -> Result<ExprIdx, ParseError> {
        let mut expr_id = self.comparison()?;
        while self.matches(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr_id = self.ast_arena.insert_expr(Expr::Binary {
                left: expr_id,
                operator,
                right,
            });
        }

        Result::Ok(expr_id)
    }

    fn comparison(&mut self) -> Result<ExprIdx, ParseError> {
        let mut expr_id = self.term()?;
        while self.matches(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr_id = self.ast_arena.insert_expr(Expr::Binary {
                left: expr_id,
                operator,
                right,
            });
        }

        Result::Ok(expr_id)
    }

    fn term(&mut self) -> Result<ExprIdx, ParseError> {
        let mut expr_id = self.factor()?;
        while self.matches(&[Minus, Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr_id = self.ast_arena.insert_expr(Expr::Binary {
                left: expr_id,
                operator,
                right,
            });
        }

        Result::Ok(expr_id)
    }

    fn factor(&mut self) -> Result<ExprIdx, ParseError> {
        let mut expr_id = self.unary()?;
        while self.matches(&[Slash, Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr_id = self.ast_arena.insert_expr(Expr::Binary {
                left: expr_id,
                operator,
                right,
            });
        }

        Result::Ok(expr_id)
    }

    fn unary(&mut self) -> Result<ExprIdx, ParseError> {
        if self.matches(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(self.ast_arena.insert_expr(Expr::Unary { operator, right }));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<ExprIdx, ParseError> {
        if self.matches(&[False]) {
            return Ok(self.ast_arena.insert_expr(Expr::Literal {
                value: RuntimeValue::Bool(false),
            }));
        }
        if self.matches(&[True]) {
            return Ok(self.ast_arena.insert_expr(Expr::Literal {
                value: RuntimeValue::Bool(true),
            }));
        }
        if self.matches(&[Nil]) {
            return Ok(self.ast_arena.insert_expr(Expr::Literal {
                value: RuntimeValue::Nil,
            }));
        }

        if self.matches(&[TokenType::Number, TokenType::String]) {
            let prev = self
                .ast_arena
                .get_token(self.previous())
                .literal
                .clone()
                .expect("Literal was not found.");

            return Ok(self.ast_arena.insert_expr(Expr::Literal { value: prev }));
        }

        if self.matches(&[Identifier]) {
            return Ok(self.ast_arena.insert_expr(Expr::Variable {
                name: self.previous(),
            }));
        }

        if self.matches(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(&RightParen, "Expect ')' after expression.")?;
            return Ok(self
                .ast_arena
                .insert_expr(Expr::Grouping { expression: expr }));
        }

        Err(ParseError::new(self.peek().clone(), "Expect expression."))
    }

    /// Consumes the current token only if it's of type `toktype` returning it,
    /// else returns an error with the `message` string
    fn consume(&mut self, toktype: &TokenType, message: &str) -> Result<TokenIdx, ParseError> {
        if self.check(toktype) {
            return Ok(self.advance());
        }

        Result::Err(ParseError::new(self.peek().clone(), message))
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            let prev = self.ast_arena.get_token(self.previous());

            if prev.toktype == Semicolon {
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

    fn declaration(&mut self) -> Result<StmtIdx, ParseError> {
        if self.matches(&[TokenType::Var]) {
            match self.var_declaration() {
                Ok(val) => return Result::Ok(val),
                Err(_err) => self.synchronize(),
            }
        }

        self.statement()
    }

    fn statement(&mut self) -> Result<StmtIdx, ParseError> {
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
            let statements = self.block()?;
            return Ok(self.ast_arena.insert_stmt(Stmt::Block { statements }));
        }

        Result::Ok(self.expression_statement()?)
    }

    fn while_statement(&mut self) -> Result<StmtIdx, ParseError> {
        self.consume(&LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expect ')' after if condition.")?;
        let body = self.statement()?;
        Ok(self.ast_arena.insert_stmt(Stmt::While { condition, body }))
    }

    fn if_statement(&mut self) -> Result<StmtIdx, ParseError> {
        self.consume(&LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;

        let else_branch = if self.matches(&[Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(self.ast_arena.insert_stmt(Stmt::If {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn block(&mut self) -> Result<Vec<StmtIdx>, ParseError> {
        let mut statements: Vec<StmtIdx> = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(&TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<StmtIdx, ParseError> {
        let expr = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after value.")?;

        Ok(self.ast_arena.insert_stmt(Stmt::Print { expr }))
    }

    fn expression_statement(&mut self) -> Result<StmtIdx, ParseError> {
        let expr = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after value.")?;

        Ok(self.ast_arena.insert_stmt(Stmt::Expression { expr }))
    }

    fn var_declaration(&mut self) -> Result<StmtIdx, ParseError> {
        let name = self.consume(&TokenType::Identifier, "Expect variable name.")?;

        let mut initializer = self.ast_arena.insert_expr(Expr::Literal {
            value: RuntimeValue::Nil,
        });

        if self.matches(&[TokenType::Equal]) {
            initializer = self.expression()?;
        }

        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(self.ast_arena.insert_stmt(Stmt::Var { name, initializer }))
    }
}
