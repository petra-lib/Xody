use std::collections::HashMap;

use crate::{
    RuntimeValue, Token,
    TokenType::{
        self, And, Bang, BangEqual, Class, Comma, Dot, Else, Equal, EqualEqual, False, For, Fun,
        Greater, GreaterEqual, Identifier, If, LeftBrace, LeftParen, Less, LessEqual, Minus, Nil,
        Or, Plus, Print, Return, RightBrace, RightParen, Semicolon, Slash, Star, Super, This, True,
        Var, While,
    },
    ast_arena::AstArena,
    errors::LexingError,
};

/// A struct representing the scanner of our interpreter
///
/// `Scanner` manages the Lexical Analysis of our input file or input commands
pub struct Scanner<'a> {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<&'static str, TokenType>,
    ast_arena: &'a mut AstArena,
}

impl<'a> Scanner<'a> {
    pub fn new(src: &str, ast_arena: &'a mut AstArena) -> Self {
        let mut s = Scanner {
            source: src.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
            keywords: HashMap::new(),
            ast_arena,
        };

        // Reserved keywords
        s.keywords.insert("and", And);
        s.keywords.insert("class", Class);
        s.keywords.insert("else", Else);
        s.keywords.insert("false", False);
        s.keywords.insert("for", For);
        s.keywords.insert("fun", Fun);
        s.keywords.insert("if", If);
        s.keywords.insert("nil", Nil);
        s.keywords.insert("or", Or);
        s.keywords.insert("print", Print);
        s.keywords.insert("return", Return);
        s.keywords.insert("super", Super);
        s.keywords.insert("this", This);
        s.keywords.insert("true", True);
        s.keywords.insert("var", Var);
        s.keywords.insert("while", While);

        s
    }

    /// Check if we're at the end of the file
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    /// Scan all tokens in the file, saving them in the "tokens" array
    pub fn scan_tokens(&mut self) -> Result<(), LexingError> {
        self.ast_arena.clear_tokens();
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.ast_arena
            .insert_token(Token::new(TokenType::EOF, String::new(), self.line));
        Ok(())
    }

    /// Parse the current token or generates a new error saved in self.error
    ///
    /// Even if an error is generated, this function will still consume the token
    fn scan_token(&mut self) -> Result<(), LexingError> {
        match self.advance() {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            ';' => self.add_token(Semicolon),
            '*' => self.add_token(Star),

            '!' => {
                let toktype = if self.next_is('=') { BangEqual } else { Bang };
                self.add_token(toktype);
            }
            '=' => {
                let toktype = if self.next_is('=') { EqualEqual } else { Equal };
                self.add_token(toktype);
            }
            '<' => {
                let toktype = if self.next_is('=') { LessEqual } else { Less };
                self.add_token(toktype);
            }
            '>' => {
                let toktype = if self.next_is('=') {
                    GreaterEqual
                } else {
                    Greater
                };
                self.add_token(toktype);
            }

            '/' => {
                // If it is a comment we skip the whole line
                if self.next_is('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    // Else we treat it as a division
                    self.add_token(Slash)
                }
            }

            // We ignore these
            ' ' | '\r' | '\t' => {}

            '\n' => self.line += 1,

            '"' => self.string()?,

            c => {
                if Self::is_digit(c) {
                    self.number();
                    return Result::Ok(());
                } else if Self::is_alpha(c) {
                    self.identifier();
                    return Result::Ok(());
                }

                return Result::Err(LexingError::new(self.line, "", "Unexpected character."));
            }
        }

        Result::Ok(())
    }

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_alpha_numeric(c: char) -> bool {
        Self::is_digit(c) || Self::is_alpha(c)
    }

    /// Takes a substring of the source file from start to end included
    /// and returns an owned version of it
    fn substring(&mut self, start: usize, end: usize) -> String {
        self.source[start..end].iter().collect()
    }

    /// Takes a token type and saves it in `self.tokens` with the
    /// corresponding text representation
    fn add_token(&mut self, toktype: TokenType) {
        let text: String = self.source[self.start..self.current].iter().collect();
        self.ast_arena
            .insert_token(Token::new(toktype, text, self.line));
    }

    fn add_token_literal(&mut self, toktype: TokenType, literal: RuntimeValue) {
        let text: String = self.source[self.start..self.current].iter().collect();
        self.ast_arena
            .insert_token(Token::new_literal(toktype, text, self.line, literal));
    }

    /// Consumes a char and returns it
    fn advance(&mut self) -> char {
        let result = self.source[self.current];
        self.current += 1;
        return result;
    }

    /// Consumes a char and returns if the char was equal to `expected`
    fn next_is(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    /// Returns the current char without consuming it
    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    // Returns the next char without consuming it
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current + 1]
    }

    /// Checks if the current token is a valid string (surrounded by ")
    ///
    /// If it's not valid, it generates an error stored in "self.error"
    fn string(&mut self) -> Result<(), LexingError> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1
            }
            self.advance();
        }

        if self.is_at_end() {
            return Result::Err(LexingError::new(self.line, "", "Unterminated string."));
        }

        // The closing "
        self.advance();

        let text = self.substring(self.start + 1, self.current - 1);
        self.add_token_literal(TokenType::String, RuntimeValue::String(text));
        Result::Ok(())
    }

    /// Checks if the current token is a valid number and parses it
    ///
    /// If it's not valid, it generates an error stored in "self.error"
    fn number(&mut self) {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        // Looking past the decimal point requires a second character of lookahead
        // since we don't want to consume the "." until we're sure there is a digit after it.
        // So we use `self.peek_next()`
        if self.peek() == '.' && Self::is_digit(self.peek_next()) {
            // Consume the "."
            self.advance();

            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }

        let numstr = self.substring(self.start, self.current);

        self.add_token_literal(
            TokenType::Number,
            RuntimeValue::Number(numstr.parse::<f64>().unwrap()),
        );
    }

    /// Checks if the current token is a valid identifier
    ///
    /// If it's not valid, it generates an error stored in "self.error"
    fn identifier(&mut self) {
        // Here we use is_alpha_numeric() since that's not the first character anymore
        // so it's allowed to be a number too
        while Self::is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.substring(self.start, self.current);

        // if the identifier matches a keyword we use that. Otherwise it's
        // just a regular user-defined identifier
        let toktype: TokenType = match self.keywords.get(text.as_str()) {
            Some(toktype) => toktype.clone(),
            None => Identifier,
        };

        self.add_token(toktype);
    }
}
