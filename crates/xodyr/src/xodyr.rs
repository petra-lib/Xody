use std::{
    cell::RefCell,
    fmt::Display,
    fs::File,
    io::{Read, Write},
    process::exit,
    rc::Rc,
};

use crate::{
    ast_arena::AstArena,
    environment::Environment,
    errors::{GenericError, XodyError},
    interpreter::Interpreter,
    parser::Parser,
    scanner::Scanner,
};

mod ast_arena;
mod environment;
mod errors;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;

#[derive(Clone, PartialEq, Debug)]
enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Number,
    String,
    Identifier,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Clone, PartialEq, Debug)]
enum RuntimeValue {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl ToString for RuntimeValue {
    fn to_string(&self) -> String {
        match self {
            RuntimeValue::Number(val) => val.to_string(),
            RuntimeValue::String(val) => val.clone(),
            RuntimeValue::Bool(val) => val.to_string(),
            RuntimeValue::Nil => "nil".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    toktype: TokenType,
    lexeme: String,
    line: usize,
    literal: Option<RuntimeValue>,
}

impl Token {
    fn new(toktype: TokenType, lexeme: String, line: usize) -> Self {
        return Self {
            toktype,
            lexeme,
            line,
            literal: None,
        };
    }

    fn new_literal(
        toktype: TokenType,
        lexeme: String,
        line: usize,
        literal: RuntimeValue,
    ) -> Token {
        return Self {
            toktype,
            lexeme,
            line,
            literal: Some(literal),
        };
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        format!("{} {}", self.toktype, self.lexeme)
    }
}

struct Xodyr {}

impl Xodyr {
    fn new() -> Self {
        Self {}
    }

    fn run(
        &mut self,
        src: &str,
        ast_arena: &mut AstArena,
        env: Rc<RefCell<Environment>>,
    ) -> Result<(), Box<dyn XodyError>> {
        Scanner::new(src, ast_arena).scan_tokens()?;
        let statements = Parser::new(ast_arena).parse()?;
        Interpreter::new(ast_arena, env).interpret(statements)?;
        Ok(())
    }
}

fn run_prompt() {
    println!("You're now in Prompt Mode, press Ctrl-c to exit");
    let stdin = std::io::stdin();
    let mut buf = String::new();

    let mut ast_arena = AstArena::new();
    let env = Rc::new(RefCell::new(Environment::new(None)));

    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();
        buf.clear();

        if let Err(err) = stdin.read_line(&mut buf) {
            GenericError::new(&err.to_string()).throw();
        }

        // let val = buf.replace("\r", "").replace("\n", "");
        if !buf.is_empty() {
            if let Err(err) = Xodyr::new().run(&buf, &mut ast_arena, env.clone()) {
                err.report();
            }
        }
    }
}

fn run_file(path: &str) {
    match File::open(path) {
        Err(err) => GenericError::new(&err.to_string()).throw(),
        Ok(mut file) => {
            let mut buf = String::new();
            if let Err(err) = file.read_to_string(&mut buf) {
                GenericError::new(&err.to_string()).throw();
            }

            // let val = buf.replace("\r", "").replace("\n", "");
            if !buf.is_empty() {
                let mut ast_arena = AstArena::new();
                let env = Rc::new(RefCell::new(Environment::new(None)));
                if let Err(err) = Xodyr::new().run(&buf, &mut ast_arena, env) {
                    err.throw();
                }
            }
        }
    };
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 2 {
        println!("Usage: {} [script]", args[0]);
        exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}
