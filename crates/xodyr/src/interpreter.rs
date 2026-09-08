use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    RuntimeValue, Token, TokenType,
    environment::Environment,
    errors::{RuntimeError, XodyError},
    expr::Expr::{self, Literal},
    stmt::Stmt,
};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new(None))),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Rc<Stmt>>) {
        for statement in statements {
            if let Err(err) = self.execute_stmt(statement) {
                err.throw()
            }
        }
    }

    fn execute_stmt(&mut self, stmt: Rc<Stmt>) -> Result<RuntimeValue, RuntimeError> {
        match (*stmt).clone() {
            Stmt::Expression { expr } => self.evaluate_expr(expr),
            Stmt::Print { expr } => self.print_expr(expr),
            Stmt::Var { name, initializer } => self.define_variable(name, initializer),
            Stmt::Block { statements } => self.execute_block(statements, self.environment.clone()),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.execute_branch(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.execute_while(condition, body),
        }
    }

    fn execute_while(
        &mut self,
        condition: Rc<Expr>,
        body: Rc<Stmt>,
    ) -> Result<RuntimeValue, RuntimeError> {
        while Self::is_truthy(self.evaluate_expr(condition.clone())?) {
            self.execute_stmt(body.clone())?;
        }
        return Ok(RuntimeValue::Nil);
    }

    fn execute_branch(
        &mut self,
        condition: Rc<Expr>,
        then_branch: Rc<Stmt>,
        else_branch: Option<Rc<Stmt>>,
    ) -> Result<RuntimeValue, RuntimeError> {
        let expression_result = self.evaluate_expr(condition)?;
        if Self::is_truthy(expression_result) {
            self.execute_stmt(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute_stmt(else_branch)?;
        }

        Ok(RuntimeValue::Nil)
    }

    fn define_variable(
        &mut self,
        name: Rc<Token>,
        initializer: Rc<Expr>,
    ) -> Result<RuntimeValue, RuntimeError> {
        let val = self.evaluate_expr(initializer)?;
        self.environment.borrow_mut().define(&name.lexeme, val);
        Result::Ok(RuntimeValue::Nil)
    }

    fn execute_block(
        &mut self,
        statements: Vec<Rc<Stmt>>,
        temp_env: Rc<RefCell<Environment>>,
    ) -> Result<RuntimeValue, RuntimeError> {
        let previous = self.environment.clone();

        self.environment = temp_env;
        for statement in statements {
            self.execute_stmt(statement)?;
        }

        self.environment = previous;
        Ok(RuntimeValue::Nil)
    }

    fn print_expr(&mut self, expr: Rc<Expr>) -> Result<RuntimeValue, RuntimeError> {
        println!("{}", self.evaluate_expr(expr)?.to_string());
        Result::Ok(RuntimeValue::Nil)
    }

    fn evaluate_expr(&mut self, expr: Rc<Expr>) -> Result<RuntimeValue, RuntimeError> {
        match (*expr).clone() {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => self.evaluate_expr(expression),
            Expr::Unary { operator, right } => self.unary_expr(operator, right),
            Literal { value } => self.literal_expr(value),
            Expr::Variable { name } => Result::Ok(self.environment.borrow().get(&name)),
            Expr::Assign { name, value } => self.assign_expr(name, value),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.logical_expr(left, operator, right),
        }
    }

    fn literal_expr(&mut self, value: Rc<RuntimeValue>) -> Result<RuntimeValue, RuntimeError> {
        Result::Ok((*value).clone())
    }

    fn logical_expr(
        &mut self,
        left: Rc<Expr>,
        operator: Rc<Token>,
        right: Rc<Expr>,
    ) -> Result<RuntimeValue, RuntimeError> {
        let left = self.evaluate_expr(left)?;
        match operator.toktype {
            TokenType::Or => {
                if Self::is_truthy(left.clone()) {
                    return Ok(left);
                }
            }
            TokenType::And => {
                if !Self::is_truthy(left.clone()) {
                    return Ok(left);
                }
            }
            _ => {
                unreachable!()
            }
        }

        self.evaluate_expr(right)
    }

    fn assign_expr(
        &mut self,
        name: Rc<Token>,
        value: Rc<Expr>,
    ) -> Result<RuntimeValue, RuntimeError> {
        let value = self.evaluate_expr(value)?;
        self.environment.borrow_mut().assign(&name, value.clone());
        Result::Ok(value)
    }

    fn unary_expr(
        &mut self,
        operator: Rc<Token>,
        right: Rc<Expr>,
    ) -> Result<RuntimeValue, RuntimeError> {
        let right = self.evaluate_expr(right)?;

        let result = match operator.toktype {
            TokenType::Minus => RuntimeValue::Number(-self.as_number(right)?),
            TokenType::Bang => RuntimeValue::Bool(!Self::is_truthy(right)),
            _ => unreachable!(),
        };

        Result::Ok(result)
    }

    fn visit_binary_expr(
        &mut self,
        left: Rc<Expr>,
        operator: Rc<Token>,
        right: Rc<Expr>,
    ) -> Result<RuntimeValue, RuntimeError> {
        let left = self.evaluate_expr(left)?;
        let right = self.evaluate_expr(right)?;

        let result = match operator.toktype {
            TokenType::Minus => {
                RuntimeValue::Number(self.as_number(left)? - self.as_number(right)?)
            }
            TokenType::Slash => {
                RuntimeValue::Number(self.as_number(left)? / self.as_number(right)?)
            }
            TokenType::Star => RuntimeValue::Number(self.as_number(left)? * self.as_number(right)?),

            TokenType::Plus => {
                if let RuntimeValue::Number(left) = left
                    && let RuntimeValue::Number(right) = right
                {
                    RuntimeValue::Number(left + right)
                } else if let RuntimeValue::String(left) = left
                    && let RuntimeValue::String(right) = right
                {
                    RuntimeValue::String(format!("{}{}", left, right))
                } else {
                    return Result::Err(RuntimeError::new("Trying to add incompatible types"));
                }
            }

            TokenType::Greater => {
                RuntimeValue::Bool(self.as_number(left)? > self.as_number(right)?)
            }
            TokenType::Less => RuntimeValue::Bool(self.as_number(left)? < self.as_number(right)?),

            TokenType::GreaterEqual => {
                RuntimeValue::Bool(self.as_number(left)? >= self.as_number(right)?)
            }
            TokenType::LessEqual => {
                RuntimeValue::Bool(self.as_number(left)? <= self.as_number(right)?)
            }

            TokenType::BangEqual => RuntimeValue::Bool(!self.is_equal(left, right)),
            TokenType::EqualEqual => RuntimeValue::Bool(self.is_equal(left, right)),

            _ => unreachable!(),
        };

        Result::Ok(result)
    }

    fn as_number(&self, runtime_type: RuntimeValue) -> Result<f64, RuntimeError> {
        match runtime_type {
            RuntimeValue::Number(num) => Result::Ok(num),
            RuntimeValue::String(_) => {
                Result::Err(RuntimeError::new("Cannot convert string to number"))
            }
            RuntimeValue::Bool(_) => {
                Result::Err(RuntimeError::new("Cannot convert boolean to number"))
            }
            RuntimeValue::Nil => {
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

    fn is_truthy(runtime_type: RuntimeValue) -> bool {
        match runtime_type {
            RuntimeValue::Nil => false,
            RuntimeValue::Bool(val) => val,
            _ => true,
        }
    }

    fn is_equal(&self, a: RuntimeValue, b: RuntimeValue) -> bool {
        if a == RuntimeValue::Nil {
            return b == RuntimeValue::Nil;
        }

        a == b
    }
}
