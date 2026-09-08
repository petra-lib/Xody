use std::collections::HashMap;

use crate::{
    RuntimeType, Token,
    errors::{RuntimeError, XodyError},
};

#[derive(Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, RuntimeType>,
}

impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Self {
        Self {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: RuntimeType) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> RuntimeType {
        if let Some(value) = self.values.get(&name.lexeme) {
            return value.clone();
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name);
        }

        RuntimeError::new(&format!("Undefined variable '{}'.", name.lexeme)).throw();
    }

    pub fn assign(&mut self, name: &Token, value: RuntimeType) {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return;
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.assign(name, value);
        }

        RuntimeError::new(&format!("Undefined variable '{}'.", name.lexeme)).throw();
    }
}
