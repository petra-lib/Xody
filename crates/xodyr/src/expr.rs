use std::rc::Rc;

use crate::{RuntimeValue, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Rc<Expr>,
        operator: Rc<Token>,
        right: Rc<Expr>,
    },
    Grouping {
        expression: Rc<Expr>,
    },
    Logical {
        left: Rc<Expr>,
        operator: Rc<Token>,
        right: Rc<Expr>,
    },
    Unary {
        operator: Rc<Token>,
        right: Rc<Expr>,
    },
    Literal {
        value: Rc<RuntimeValue>,
    },
    Variable {
        name: Rc<Token>,
    },
    Assign {
        name: Rc<Token>,
        value: Rc<Expr>,
    },
}

impl Expr {
    pub fn new_binary(left: Rc<Expr>, operator: Rc<Token>, right: Rc<Expr>) -> Rc<Expr> {
        Rc::new(Expr::Binary {
            left,
            operator,
            right,
        })
    }

    pub fn new_grouping(expression: Rc<Expr>) -> Rc<Expr> {
        Rc::new(Expr::Grouping { expression })
    }

    pub fn new_logical(left: Rc<Expr>, operator: Rc<Token>, right: Rc<Expr>) -> Rc<Expr> {
        Rc::new(Expr::Logical {
            left,
            operator,
            right,
        })
    }

    pub fn new_unary(operator: Rc<Token>, right: Rc<Expr>) -> Rc<Expr> {
        Rc::new(Expr::Unary { operator, right })
    }

    pub fn new_literal(value: Rc<RuntimeValue>) -> Rc<Expr> {
        Rc::new(Expr::Literal { value })
    }

    pub fn new_variable(name: Rc<Token>) -> Rc<Expr> {
        Rc::new(Expr::Variable { name })
    }

    pub fn new_assign(name: Rc<Token>, value: Rc<Expr>) -> Rc<Expr> {
        Rc::new(Expr::Assign { name, value })
    }
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
            Expr::Assign { name, value } => parenthesize!("assign", name, value),
            Expr::Logical {
                left,
                operator,
                right,
            } => parenthesize!(operator.lexeme, left, right),
        }
    }
}
