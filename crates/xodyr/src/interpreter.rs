use crate::{
    RuntimeType, Token, TokenType,
    environment::Environment,
    errors::{RuntimeError, XodyError},
    expr::Expr::{self, Literal},
    stmt::Stmt,
};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(None),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            if let Err(err) = self.execute_stmt(statement) {
                err.throw()
            }
        }
    }

    fn execute_stmt(&mut self, stmt: Stmt) -> Result<RuntimeType, RuntimeError> {
        match stmt {
            Stmt::Expression { expr } => self.evaluate_expr(*expr),
            Stmt::Print { expr } => self.print_expr(*expr),
            Stmt::Var { name, initializer } => self.define_variable(name, *initializer),
            Stmt::Block { statements } => self.execute_block(statements, self.environment.clone()),
        }
    }

    fn define_variable(
        &mut self,
        name: Token,
        initializer: Expr,
    ) -> Result<RuntimeType, RuntimeError> {
        let mut val = RuntimeType::Nil;

        match initializer {
            Expr::Literal { value } => {
                if value != RuntimeType::Nil {
                    val = value;
                }
            }
            _ => {}
        }

        self.environment.define(&name.lexeme, val);
        Result::Ok(RuntimeType::Nil)
    }

    fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        temp_env: Environment,
    ) -> Result<RuntimeType, RuntimeError> {
        let previous = self.environment.clone();

        self.environment = temp_env;
        for statement in statements {
            self.execute_stmt(statement)?;
        }

        self.environment = previous;
        Ok(RuntimeType::Nil)
    }

    fn print_expr(&mut self, expr: Expr) -> Result<RuntimeType, RuntimeError> {
        println!("{}", self.evaluate_expr(expr)?.to_string());
        Result::Ok(RuntimeType::Nil)
    }

    fn evaluate_expr(&mut self, expr: Expr) -> Result<RuntimeType, RuntimeError> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => self.evaluate_expr(*expression),
            Expr::Unary { operator, right } => self.visit_unary_expr(operator, right),
            Literal { value } => Result::Ok(value),
            Expr::Variable { name } => Result::Ok(self.environment.get(&name)),
            Expr::Assign { name, value } => self.visit_assign_expr(name, *value),
        }
    }

    fn visit_assign_expr(&mut self, name: Token, value: Expr) -> Result<RuntimeType, RuntimeError> {
        let value = self.evaluate_expr(value)?;
        self.environment.assign(&name, value.clone());
        Result::Ok(value)
    }

    fn visit_unary_expr(
        &mut self,
        operator: Token,
        right: Box<Expr>,
    ) -> Result<RuntimeType, RuntimeError> {
        let right = self.evaluate_expr(*right)?;

        let result = match operator.toktype {
            TokenType::Minus => RuntimeType::Number(-self.as_number(right)?),
            TokenType::Bang => RuntimeType::Bool(!self.is_truthy(right)),
            _ => unreachable!(),
        };

        Result::Ok(result)
    }

    fn visit_binary_expr(
        &mut self,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    ) -> Result<RuntimeType, RuntimeError> {
        let left = self.evaluate_expr(*left)?;
        let right = self.evaluate_expr(*right)?;

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
