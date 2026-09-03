use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
    process::exit,
};

mod scanner;

#[derive(Clone)]
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
    Identifier,
    Str { value: String },
    Num { value: f64 },

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

struct Token {
    toktype: TokenType,
    lexeme: String,
    line: usize,
}

impl Token {
    fn new(toktype: TokenType, lexeme: String, line: usize) -> Self {
        return Self {
            toktype,
            lexeme,
            line,
        };
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        format!("{} {}", self.toktype, self.lexeme)
    }
}

struct XodyErr {
    line: usize,
    where_: String,
    message: String,
}

impl XodyErr {
    fn new(line: usize, where_: &str, message: &str) -> Self {
        Self {
            line,
            where_: where_.to_owned(),
            message: message.to_owned(),
        }
    }

    fn new_line_msg(line: usize, message: &str) -> Self {
        Self::new(line, "", message)
    }

    fn throw(&self) {
        eprintln!(
            "[line {}] Error {}: {}",
            self.line, self.where_, self.message
        );
    }
}

struct Parser {}

impl Parser {
    fn new() -> Self {
        Self {}
    }

    fn run(&self, src: &str) -> Result<(), XodyErr> {
        for token in src.chars() {
            println!("{}", token);
        }

        Result::Ok(())
    }
}

fn die(err: &str) {
    eprintln!("[ERROR]: {}", err);
    exit(-1);
}

fn run_prompt(p: &mut Parser) {
    println!("You're now in Prompt Mode, press Ctrl-c to exit");
    let stdin = std::io::stdin();
    let mut buf = String::new();
    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();
        buf.clear();
        if let Err(err) = stdin.read_line(&mut buf) {
            die(&err.to_string());
        }

        let val = buf.replace("\r", "").replace("\n", "");
        if !val.is_empty() {
            let _result = p.run(&val);
        }
    }
}

fn run_file(p: &Parser, path: &str) {
    match File::open(path) {
        Err(err) => die(&err.to_string()),
        Ok(mut file) => {
            let mut buf = String::new();
            if let Err(err) = file.read_to_string(&mut buf) {
                die(&err.to_string());
            }

            let val = buf.replace("\r", "").replace("\n", "");
            if !val.is_empty() {
                let result = p.run(&val);
                if let Err(_err) = result {
                    exit(65);
                }
            }
        }
    };
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut parser = Parser::new();
    if args.len() > 2 {
        println!("Usage: {} [script]", args[0]);
        exit(64);
    } else if args.len() == 2 {
        run_file(&parser, &args[1]);
    } else {
        run_prompt(&mut parser);
    }
}
