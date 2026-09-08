use crate::{
    RuntimeValue,
    ast_arena::{ExprIdx, TokenIdx},
};

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: ExprIdx,
        operator: TokenIdx,
        right: ExprIdx,
    },
    Grouping {
        expression: ExprIdx,
    },
    Logical {
        left: ExprIdx,
        operator: TokenIdx,
        right: ExprIdx,
    },
    Unary {
        operator: TokenIdx,
        right: ExprIdx,
    },
    Literal {
        value: RuntimeValue,
    },
    Variable {
        name: TokenIdx,
    },
    Assign {
        name: TokenIdx,
        value: ExprIdx,
    },
}
