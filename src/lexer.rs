use std::fs::File;
use std::io::prelude::*;
use std::str::Chars;

use crate::syntax::{Token, TokenInfo, TokenKind};

#[cfg(test)]
mod lexer_test {
    use super::*;

    fn dump(tokens: Vec<Token>) {
        for t in &tokens {
            print!("{:?} ", t);
        }
        print!("\n");
    }

    #[test]
    fn test1() {
        let mut lexer = Lexer::from_file("src/test/test.txt").unwrap();
        // dump(lexer.lex().unwrap());
        // test with --nocapture arg and see output
    }
    #[test]
    fn test2() {
        let mut lexer = Lexer::from_file("src/test/test_parser.txt").unwrap();
        // dump(lexer.lex().unwrap());
        // test with --nocapture arg and see output
    }
}

struct Eater<'a> {
    input: &'a str,
    input_iter: std::iter::Peekable<Chars<'a>>,
    was_newline: bool,
    cc: char,

    row: usize,
    col: usize,
}

impl<'a> Eater<'a> {
    pub fn from_str(input: &str) -> Eater {
        Eater {
            input,
            input_iter: input.chars().peekable(),
            was_newline: false,
            cc: ' ',
            row: 1,
            col: 0,
        }
    }

    fn next_char(&mut self) {
        if self.was_newline {
            self.col = 0;
            self.row += 1;
            self.was_newline = false;
        }
        match self.input_iter.next() {
            Some('\n') => {
                self.was_newline = true;
                self.col += 1;
                self.cc = '\n';
            }
            Some(c) => {
                self.col += 1;
                self.cc = c;
            }

            None => self.cc = '\0',
        }
    }

    fn skip_white(&mut self) {
        loop {
            if !self.cc.is_whitespace() {
                break;
            }
            self.next_char();
        }
    }

    fn eat_alnum_dump(&mut self) -> (String, usize, usize) {
        let mut tmp = String::new();
        let e_row;
        let e_col;
        loop {
            if !self.cc.is_alphanumeric() {
                e_row = self.row;
                e_col = self.col - 1;
                break;
            }
            tmp.push(self.cc);
            self.next_char();
        }
        (tmp, e_row, e_col)
    }

    fn eat_num_dump(&mut self) -> (String, usize, usize) {
        let mut tmp = String::new();
        let e_row;
        let e_col;
        loop {
            if !self.cc.is_numeric() {
                e_row = self.row;
                e_col = self.col - 1;
                break;
            }
            tmp.push(self.cc);
            self.next_char();
        }
        (tmp, e_row, e_col)
    }

    pub fn eat_token_dump(&mut self) -> Result<Token, &str> {
        self.skip_white();
        match self.cc {
            c if c.is_alphabetic() => {
                let mut id = String::new();
                id.push(c);
                let s_col = self.col;
                let s_row = self.row;
                self.next_char();
                let (rest, e_row, e_col) = self.eat_alnum_dump();
                let info = TokenInfo {
                    s_col,
                    s_row,
                    e_col,
                    e_row,
                };
                let id = format!("{}{}", id, rest);
                let kind = match id.as_str() {
                    "fn" => TokenKind::Func,
                    "function" => TokenKind::FuncAnon,
                    "Fn" => TokenKind::FuncType,
                    "unit" => TokenKind::UnitVal,
                    "Unit" => TokenKind::UnitType,
                    "I32" => TokenKind::I32,
                    _ => TokenKind::Ident(id),
                };
                Ok(Token { kind, info })
            }

            c if c.is_numeric() => {
                let s_col = self.col;
                let s_row = self.row;
                let mut num = String::new();
                num.push(c);
                self.next_char();
                let (rest, e_row, e_col) = self.eat_num_dump();
                let info = TokenInfo {
                    s_col,
                    s_row,
                    e_col,
                    e_row,
                };
                let num = format!("{}{}", num, rest);
                let kind = TokenKind::Num(num);
                Ok(Token { kind, info })
            }

            '(' => {
                let info = TokenInfo {
                    s_col: self.col,
                    s_row: self.row,
                    e_col: self.col,
                    e_row: self.row,
                };
                self.next_char();
                Ok(Token {
                    kind: TokenKind::LParen,
                    info,
                })
            }

            ')' => {
                let info = TokenInfo {
                    s_col: self.col,
                    s_row: self.row,
                    e_col: self.col,
                    e_row: self.row,
                };
                self.next_char();
                Ok(Token {
                    kind: TokenKind::RParen,
                    info,
                })
            }

            '{' => {
                let info = TokenInfo {
                    s_col: self.col,
                    s_row: self.row,
                    e_col: self.col,
                    e_row: self.row,
                };
                self.next_char();
                Ok(Token {
                    kind: TokenKind::LBrace,
                    info,
                })
            }

            '}' => {
                let info = TokenInfo {
                    s_col: self.col,
                    s_row: self.row,
                    e_col: self.col,
                    e_row: self.row,
                };
                self.next_char();
                Ok(Token {
                    kind: TokenKind::RBrace,
                    info,
                })
            }

            ',' => {
                let info = TokenInfo {
                    s_col: self.col,
                    s_row: self.row,
                    e_col: self.col,
                    e_row: self.row,
                };
                self.next_char();
                Ok(Token {
                    kind: TokenKind::Comma,
                    info,
                })
            }

            ':' => {
                let info = TokenInfo {
                    s_col: self.col,
                    s_row: self.row,
                    e_col: self.col,
                    e_row: self.row,
                };
                self.next_char();
                Ok(Token {
                    kind: TokenKind::Colon,
                    info,
                })
            }

            ';' => {
                let info = TokenInfo {
                    s_col: self.col,
                    s_row: self.row,
                    e_col: self.col,
                    e_row: self.row,
                };
                self.next_char();
                Ok(Token {
                    kind: TokenKind::SemiColon,
                    info,
                })
            }
            '-' => {
                let (s_col,s_row) = (self.col,self.row);
                self.next_char();
                let e_col;
                let e_row;
                let kind = match self.cc {
                    '>' => {
                        e_col = self.col;
                        e_row = self.row;
                        self.next_char();
                        TokenKind::Arrow
                    }
                    _ => return Err("いやARROWじゃないんかい！（痛烈な突っ込み）")
                };
                Ok(Token{kind,info:TokenInfo{s_col,s_row,e_col,e_row}})
            }

            '\0' => {
                let info = TokenInfo {
                    s_col: self.col,
                    s_row: self.row,
                    e_col: self.col,
                    e_row: self.row,
                };
                self.next_char();
                Ok(Token {
                    kind: TokenKind::EOF,
                    info,
                })
            }
            _ => Err("Error: found unrecognized charactor"),
        }
    }
}

pub struct Lexer {
    buffer: String,
}

impl Lexer {
    pub fn from_file(fname: &str) -> Result<Lexer, std::io::Error> {
        let mut file = File::open(fname)?;
        let mut tmp_str = String::new();
        file.read_to_string(&mut tmp_str)?;
        Ok(Lexer { buffer: tmp_str })
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, &str> {
        let mut tokens = Vec::new();
        let mut eater = Eater::from_str(self.buffer.as_str());
        loop {
            match eater.eat_token_dump() {
                Ok(t) => match t.kind {
                    TokenKind::EOF => break,
                    _ => tokens.push(t),
                },
                Err(s) => {
                    println!("{}", s);
                    return Err("Failed to lex");
                }
            }
        }
        Ok(tokens)
    }
}
