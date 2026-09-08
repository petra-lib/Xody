use std::{cell::RefCell, rc::Rc};

use crate::{
    RuntimeValue, TokenType,
    ast_arena::{AstArena, ExprIdx, StmtIdx, TokenIdx},
    environment::Environment,
    errors::RuntimeError,
    expr::Expr::{self, Literal},
    stmt::Stmt,
};

pub struct Interpreter<'a> {
    environment: Rc<RefCell<Environment>>,
    ast_arena: &'a AstArena,
}

impl<'a> Interpreter<'a> {
    pub fn new(ast_arena: &'a AstArena, environment: Rc<RefCell<Environment>>) -> Self {
        Self {
            environment,
            ast_arena,
        }
    }

    pub fn interpret(&mut self, statements: Vec<StmtIdx>) -> Result<(), RuntimeError> {
        for statement in statements {
            self.execute_stmt(statement)?;
        }
        Ok(())
    }

    fn execute_stmt(&mut self, stmt: StmtIdx) -> Result<RuntimeValue, RuntimeError> {
        match self.ast_arena.get_stmt(stmt) {
            Stmt::Expression { expr } => self.evaluate_expr(*expr),
            Stmt::Print { expr } => self.print_expr(*expr),
            Stmt::Println { expr } => self.println_expr(*expr),
            Stmt::Var { name, initializer } => self.define_variable(*name, *initializer),
            Stmt::Block { statements } => {
                let new_env = Rc::new(RefCell::new(Environment::new(Some(
                    self.environment.clone(),
                ))));
                self.execute_block(statements, new_env)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.execute_branch(*condition, *then_branch, *else_branch),
            Stmt::While { condition, body } => self.execute_while(*condition, *body),
        }
    }

    fn execute_while(
        &mut self,
        condition: ExprIdx,
        body: StmtIdx,
    ) -> Result<RuntimeValue, RuntimeError> {
        while Self::is_truthy(self.evaluate_expr(condition)?) {
            self.execute_stmt(body)?;
        }
        return Ok(RuntimeValue::Nil);
    }

    fn execute_branch(
        &mut self,
        condition: ExprIdx,
        then_branch: StmtIdx,
        else_branch: Option<StmtIdx>,
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
        name: TokenIdx,
        initializer: ExprIdx,
    ) -> Result<RuntimeValue, RuntimeError> {
        let val = self.evaluate_expr(initializer)?;
        let name = self.ast_arena.get_token(name);
        self.environment.borrow_mut().define(&name.lexeme, val);
        Result::Ok(RuntimeValue::Nil)
    }

    fn execute_block(
        &mut self,
        statements: &Vec<StmtIdx>,
        temp_env: Rc<RefCell<Environment>>,
    ) -> Result<RuntimeValue, RuntimeError> {
        let previous = self.environment.clone();

        self.environment = temp_env;
        for statement in statements {
            self.execute_stmt(*statement)?;
        }

        self.environment = previous;
        Ok(RuntimeValue::Nil)
    }

    fn print_expr(&mut self, expr: ExprIdx) -> Result<RuntimeValue, RuntimeError> {
        print!("{}", self.evaluate_expr(expr)?.to_string());
        Result::Ok(RuntimeValue::Nil)
    }

    fn println_expr(&mut self, expr: ExprIdx) -> Result<RuntimeValue, RuntimeError> {
        println!("{}", self.evaluate_expr(expr)?.to_string());
        Result::Ok(RuntimeValue::Nil)
    }

    fn evaluate_expr(&mut self, expr_id: ExprIdx) -> Result<RuntimeValue, RuntimeError> {
        match self.ast_arena.get_expr(expr_id) {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(*left, *operator, *right),
            Expr::Grouping { expression } => self.evaluate_expr(*expression),
            Expr::Unary { operator, right } => self.unary_expr(*operator, *right),
            Literal { value } => self.literal_expr(value.clone()),
            Expr::Variable { name } => Result::Ok(
                self.environment
                    .borrow()
                    .get(self.ast_arena.get_token(*name))?,
            ),
            Expr::Assign { name, value } => self.assign_expr(*name, *value),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.logical_expr(*left, *operator, *right),
        }
    }

    fn literal_expr(&mut self, value: RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
        Result::Ok(value)
    }

    fn logical_expr(
        &mut self,
        left: ExprIdx,
        operator: TokenIdx,
        right: ExprIdx,
    ) -> Result<RuntimeValue, RuntimeError> {
        let left = self.evaluate_expr(left)?;
        let operator = self.ast_arena.get_token(operator);
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
        name: TokenIdx,
        value: ExprIdx,
    ) -> Result<RuntimeValue, RuntimeError> {
        let value = self.evaluate_expr(value)?;
        let name = self.ast_arena.get_token(name);
        self.environment.borrow_mut().assign(&name, value.clone())?;
        Result::Ok(value)
    }

    fn unary_expr(
        &mut self,
        operator: TokenIdx,
        right: ExprIdx,
    ) -> Result<RuntimeValue, RuntimeError> {
        let right = self.evaluate_expr(right)?;
        let operator = self.ast_arena.get_token(operator);

        let result = match operator.toktype {
            TokenType::Minus => RuntimeValue::Number(-self.as_number(right)?),
            TokenType::Bang => RuntimeValue::Bool(!Self::is_truthy(right)),
            _ => unreachable!(),
        };

        Result::Ok(result)
    }

    fn visit_binary_expr(
        &mut self,
        left: ExprIdx,
        operator: TokenIdx,
        right: ExprIdx,
    ) -> Result<RuntimeValue, RuntimeError> {
        let left = self.evaluate_expr(left)?;
        let right = self.evaluate_expr(right)?;

        let operator = self.ast_arena.get_token(operator);
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
                } else if self.is_string(&left) || self.is_string(&right) {
                    RuntimeValue::String(format!(
                        "{}{}",
                        self.as_string(left)?,
                        self.as_string(right)?
                    ))
                } else {
                    return Result::Err(RuntimeError::new(&format!(
                        "Trying to add incompatible types: {} {} {}",
                        self.as_string(left)?,
                        operator.lexeme,
                        self.as_string(right)?
                    )));
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

    fn as_string(&self, runtime_type: RuntimeValue) -> Result<String, RuntimeError> {
        match runtime_type {
            RuntimeValue::Number(num) => Ok(num.to_string()),
            RuntimeValue::String(val) => Ok(val),
            RuntimeValue::Bool(val) => Ok(val.to_string()),
            RuntimeValue::Nil => Ok("nil".to_string()),
        }
    }

    fn is_string(&self, runtime_type: &RuntimeValue) -> bool {
        match runtime_type {
            RuntimeValue::String(_) => true,
            _ => false,
        }
    }

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
