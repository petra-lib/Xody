use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
    process::exit,
    rc::Rc,
};

use crate::{
    errors::{GenericError, XodyError},
    interpreter::Interpreter,
    parser::Parser,
    scanner::Scanner,
};

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

impl RuntimeValue {
    pub fn new_number(val: f64) -> Rc<RuntimeValue> {
        Rc::new(RuntimeValue::Number(val))
    }

    pub fn new_string(val: String) -> Rc<RuntimeValue> {
        Rc::new(RuntimeValue::String(val))
    }

    pub fn new_bool(val: bool) -> Rc<RuntimeValue> {
        Rc::new(RuntimeValue::Bool(val))
    }

    pub fn new_nil() -> Rc<RuntimeValue> {
        Rc::new(RuntimeValue::Nil)
    }
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
struct Token {
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

    fn run(&self, src: &str) {
        let mut scan = Scanner::new(src);
        let tokens = scan.scan_tokens();
        let statements = Parser::new(tokens).parse();
        Interpreter::new().interpret(statements);
    }
}

fn die(err: &str) {
    eprintln!("[ERROR]: {}", err);
    exit(-1);
}

fn run_prompt() {
    println!("You're now in Prompt Mode, press Ctrl-c to exit");
    let stdin = std::io::stdin();
    let mut buf = String::new();
    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();
        buf.clear();

        if let Err(err) = stdin.read_line(&mut buf) {
            GenericError::new(&err.to_string()).throw();
        }

        // let val = buf.replace("\r", "").replace("\n", "");
        if !buf.is_empty() {
            let _result = Xodyr::new().run(&buf);
        }
    }
}

fn run_file(path: &str) {
    match File::open(path) {
        Err(err) => die(&err.to_string()),
        Ok(mut file) => {
            let mut buf = String::new();
            if let Err(err) = file.read_to_string(&mut buf) {
                die(&err.to_string());
            }

            // let val = buf.replace("\r", "").replace("\n", "");
            if !buf.is_empty() {
                Xodyr::new().run(&buf);
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
