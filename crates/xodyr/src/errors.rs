use std::process::exit;

use crate::{Token, TokenType};

pub trait XodyError {
    fn throw(&self) -> !;
    fn report(&self);
}

pub struct GenericError {
    message: String,
}

impl GenericError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl XodyError for GenericError {
    fn throw(&self) -> ! {
        self.report();
        exit(65);
    }

    fn report(&self) {
        eprintln!("Error: {}", self.message);
    }
}

pub struct LexingError {
    line: usize,
    where_: String,
    message: String,
}

impl LexingError {
    pub fn new(line: usize, where_: &str, message: &str) -> Self {
        Self {
            line,
            where_: where_.to_string(),
            message: message.to_string(),
        }
    }
}

impl XodyError for LexingError {
    fn throw(&self) -> ! {
        self.report();
        exit(65);
    }

    fn report(&self) {
        eprintln!(
            "[line {}] Error {}: {}",
            self.line, self.where_, self.message
        );
    }
}

pub struct ParseError {
    token: Token,
    message: String,
}

impl ParseError {
    pub fn new(token: Token, message: &str) -> Self {
        Self {
            token,
            message: message.to_string(),
        }
    }
}

impl XodyError for ParseError {
    fn throw(&self) -> ! {
        self.report();
        exit(65);
    }

    fn report(&self) {
        match self.token.toktype {
            TokenType::EOF => {
                eprintln!(
                    "[line {}] Error at end: {}",
                    self.token.line, self.token.lexeme
                );
            }
            _ => {
                eprintln!(
                    "[line {}] Error at {}: {}",
                    self.token.line, self.token.lexeme, self.token.lexeme
                );
            }
        }
    }
}
