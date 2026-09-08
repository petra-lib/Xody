use crate::ast_arena::{ExprIdx, StmtIdx, TokenIdx};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression {
        expr: ExprIdx,
    },
    If {
        condition: ExprIdx,
        then_branch: StmtIdx,
        else_branch: Option<StmtIdx>,
    },
    Print {
        expr: ExprIdx,
    },
    Println {
        expr: ExprIdx,
    },
    Var {
        name: TokenIdx,
        initializer: ExprIdx,
    },
    While {
        condition: ExprIdx,
        body: StmtIdx,
    },
    Block {
        statements: Vec<StmtIdx>,
    },
}
