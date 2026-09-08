use crate::{Token, expr::Expr};

pub enum Stmt {
    Expression { expr: Box<Expr> },
    Print { expr: Box<Expr> },
    Var { name: Token, initializer: Box<Expr> },
    Block { statements: Vec<Stmt> },
}
