use crate::{
    RuntimeType, Token, TokenType,
    errors::RuntimeError,
    expr::Expr::{self, Literal},
};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, expr: Expr) -> Result<RuntimeType, RuntimeError> {
        self.evaluate(expr)
    }

    pub fn evaluate(&self, expr: Expr) -> Result<RuntimeType, RuntimeError> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => self.evaluate(*expression),
            Expr::Unary { operator, right } => self.visit_unary_expr(operator, right),
            Literal { value } => Result::Ok(value),
        }
    }

    fn visit_unary_expr(
        &self,
        operator: Token,
        right: Box<Expr>,
    ) -> Result<RuntimeType, RuntimeError> {
        let right = self.evaluate(*right)?;

        let result = match operator.toktype {
            TokenType::Minus => RuntimeType::Number(-self.as_number(right)?),
            TokenType::Bang => RuntimeType::Bool(!self.is_truthy(right)),
            _ => unreachable!(),
        };

        Result::Ok(result)
    }

    fn visit_binary_expr(
        &self,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    ) -> Result<RuntimeType, RuntimeError> {
        let left = self.evaluate(*left)?;
        let right = self.evaluate(*right)?;

        let result = match operator.toktype {
            TokenType::Minus => RuntimeType::Number(self.as_number(left)? - self.as_number(right)?),
            TokenType::Slash => RuntimeType::Number(self.as_number(left)? / self.as_number(right)?),
            TokenType::Star => RuntimeType::Number(self.as_number(left)? * self.as_number(right)?),

            TokenType::Plus => {
                if let RuntimeType::Number(left) = left
                    && let RuntimeType::Number(right) = right
                {
                    RuntimeType::Number(left + right)
                } else if let RuntimeType::String(left) = left
                    && let RuntimeType::String(right) = right
                {
                    RuntimeType::String(format!("{}{}", left, right))
                } else {
                    return Result::Err(RuntimeError::new("Trying to add incompatible types"));
                }
            }

            TokenType::Greater => RuntimeType::Bool(self.as_number(left)? > self.as_number(right)?),
            TokenType::Less => RuntimeType::Bool(self.as_number(left)? < self.as_number(right)?),

            TokenType::GreaterEqual => {
                RuntimeType::Bool(self.as_number(left)? >= self.as_number(right)?)
            }
            TokenType::LessEqual => {
                RuntimeType::Bool(self.as_number(left)? <= self.as_number(right)?)
            }

            TokenType::BangEqual => RuntimeType::Bool(!self.is_equal(left, right)),
            TokenType::EqualEqual => RuntimeType::Bool(self.is_equal(left, right)),

            _ => unreachable!(),
        };

        Result::Ok(result)
    }

    fn as_number(&self, runtime_type: RuntimeType) -> Result<f64, RuntimeError> {
        match runtime_type {
            RuntimeType::Number(num) => Result::Ok(num),
            RuntimeType::String(_) => {
                Result::Err(RuntimeError::new("Cannot convert string to number"))
            }
            RuntimeType::Bool(_) => {
                Result::Err(RuntimeError::new("Cannot convert boolean to number"))
            }
            RuntimeType::Nil => {
                Result::Err(RuntimeError::new("Cannot convert Nil value to number"))
            }
        }
    }

    // fn as_string(&self, runtime_type: RuntimeType) -> Result<String, RuntimeError> {
    //     match runtime_type {
    //         RuntimeType::String(str) => Result::Ok(str),
    //         RuntimeType::Number(_) => {
    //             Result::Err(RuntimeError::new("Cannot convert number to string"))
    //         }
    //         RuntimeType::Bool(_) => {
    //             Result::Err(RuntimeError::new("Cannot convert boolean to String"))
    //         }
    //         RuntimeType::Nil => {
    //             Result::Err(RuntimeError::new("Cannot convert Nil value to String"))
    //         }
    //     }
    // }

    fn is_truthy(&self, runtime_type: RuntimeType) -> bool {
        match runtime_type {
            RuntimeType::Nil => false,
            RuntimeType::Bool(val) => val,
            _ => true,
        }
    }

    fn is_equal(&self, a: RuntimeType, b: RuntimeType) -> bool {
        if a == RuntimeType::Nil {
            return b == RuntimeType::Nil;
        }

        a == b
    }
}
