use std::rc::Rc;

use crate::{Token, expr::Expr};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression {
        expr: Rc<Expr>,
    },
    If {
        condition: Rc<Expr>,
        then_branch: Rc<Stmt>,
        else_branch: Option<Rc<Stmt>>,
    },
    Print {
        expr: Rc<Expr>,
    },
    Var {
        name: Rc<Token>,
        initializer: Rc<Expr>,
    },
    While {
        condition: Rc<Expr>,
        body: Rc<Stmt>,
    },
    Block {
        statements: Vec<Rc<Stmt>>,
    },
}
