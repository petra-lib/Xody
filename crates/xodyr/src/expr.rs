use crate::{RuntimeType, Token};

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Literal {
        value: RuntimeType,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
}

macro_rules! parenthesize {
    ($name: expr, $($exprs: expr),*) => {{
        let mut output = String::new();
        output += "(";
        output += &$name;
        $(
            output += " ";
            output += &(*$exprs).to_string();
        )*
        output += ")";
        output
    }};
}

impl ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => parenthesize!(operator.lexeme, left, right),
            Expr::Grouping { expression } => parenthesize!("group", expression),
            Expr::Unary { operator, right } => parenthesize!(operator.lexeme, right),
            Expr::Literal { value } => value.to_string(),
            Expr::Variable { name } => name.to_string(),
            Expr::Assign { name: _name, value } => value.to_string(),
        }
    }
}
