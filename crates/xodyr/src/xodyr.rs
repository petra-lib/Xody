use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
    process::exit,
};

use crate::{
    errors::{GenericError, LexingError, XodyError},
    expr::Expr,
    parser::{AstPrinter, Parser},
    scanner::Scanner,
};

mod errors;
mod expr;
mod parser;
mod scanner;

#[derive(Clone, PartialEq)]
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

#[derive(Clone, PartialEq)]
enum RuntimeType {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Clone)]
struct Token {
    toktype: TokenType,
    lexeme: String,
    line: usize,
    literal: Option<RuntimeType>,
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

    fn new_literal(toktype: TokenType, lexeme: String, line: usize, literal: RuntimeType) -> Token {
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
        match scan.scan_tokens() {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);

                match parser.parse() {
                    Ok(expr) => AstPrinter::print(expr),
                    Err(err) => err.report(),
                }
            }
            Err(error) => {
                error.report();
            }
        }
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

        let val = buf.replace("\r", "").replace("\n", "");
        if !val.is_empty() {
            let _result = Xodyr::new().run(&val);
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

            let val = buf.replace("\r", "").replace("\n", "");
            if !val.is_empty() {
                Xodyr::new().run(&val);
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
