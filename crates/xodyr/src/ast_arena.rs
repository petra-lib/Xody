use crate::{Token, expr::Expr, stmt::Stmt};

#[derive(Debug, Clone, Copy)]
pub struct TokenIdx(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct ExprIdx(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct StmtIdx(pub usize);

pub struct AstArena {
    tokens: Vec<Token>,
    expressions: Vec<Expr>,
    statements: Vec<Stmt>,
}

impl AstArena {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            expressions: Vec::new(),
            statements: Vec::new(),
        }
    }

    pub fn clear_tokens(&mut self) {
        self.tokens.clear();
    }
    pub fn clear_expr(&mut self) {
        self.expressions.clear();
    }
    pub fn clear_stmt(&mut self) {
        self.statements.clear();
    }

    pub fn insert_token(&mut self, t: Token) -> TokenIdx {
        let index = self.tokens.len();
        self.tokens.push(t);
        TokenIdx(index)
    }

    pub fn insert_expr(&mut self, e: Expr) -> ExprIdx {
        let index = self.expressions.len();
        self.expressions.push(e);
        ExprIdx(index)
    }

    pub fn insert_stmt(&mut self, s: Stmt) -> StmtIdx {
        let index = self.statements.len();
        self.statements.push(s);
        StmtIdx(index)
    }

    pub fn get_token(&self, id: TokenIdx) -> &Token {
        &self.tokens[id.0]
    }

    pub fn get_expr(&self, id: ExprIdx) -> &Expr {
        &self.expressions[id.0]
    }

    pub fn get_stmt(&self, id: StmtIdx) -> &Stmt {
        &self.statements[id.0]
    }

    // pub fn get_token_mut(&mut self, id: TokenIdx) -> &mut Token {
    //     &mut self.tokens[id.0]
    // }

    // pub fn get_expr_mut(&mut self, id: ExprIdx) -> &mut Expr {
    //     &mut self.expressions[id.0]
    // }

    // pub fn get_stmt_mut(&mut self, id: StmtIdx) -> &mut Stmt {
    //     &mut self.statements[id.0]
    // }
}
