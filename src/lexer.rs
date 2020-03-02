use std::fs::File;
use std::io::prelude::*;
use std::str::Chars;

use crate::syntax::Token;

#[cfg(test)]
mod lexer_test {
    use super::*;

    fn dump(tokens: Vec<Token>) {
        for t in &tokens {
            print!("{:?} ", t);
        }
        print!("\n");
    }

    fn test1() {
        let mut lexer = Lexer::from_file("src/test/test.txt").unwrap();
        dump(lexer.lex().unwrap());
        // test with --nocapture arg and see output
    }
}

// struct TokenInfo{
//     s_col : usize,
//     s_row : usize,
//     e_col : usize,
//     e_row : usize,
// }

struct Eater<'a> {
    input: &'a str,
    input_iter: std::iter::Peekable<Chars<'a>>,
    cc: char,
}

impl<'a> Eater<'a> {
    pub fn from_str(input: &str) -> Eater {
        Eater {
            input,
            input_iter: input.chars().peekable(),
            cc: ' ',
        }
    }

    fn next_char(&mut self) {
        match self.input_iter.next() {
            Some(c) => {
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

    fn eat_alnum_dump(&mut self) -> String {
        let mut tmp = String::new();
        loop {
            if !self.cc.is_alphanumeric() {
                break;
            }
            tmp.push(self.cc);
            self.next_char();
        }
        tmp
    }

    fn eat_num_dump(&mut self) -> String {
        let mut tmp = String::new();
        loop {
            if !self.cc.is_numeric() {
                break;
            }
            tmp.push(self.cc);
            self.next_char();
        }
        tmp
    }

    pub fn eat_token_dump(&mut self) -> Result<Token, &str> {
        self.skip_white();
        match self.cc {
            c if c.is_alphabetic() => {
                let mut tmp = String::new();
                tmp.push(c);
                self.next_char();
                let tmp = format!("{}{}", tmp, self.eat_alnum_dump());
                match tmp.as_str() {
                    "fn" => Ok(Token::Func),
                    "function" => Ok(Token::FuncAnon),
                    "unit" => Ok(Token::UnitVal),
                    "Unit" => Ok(Token::UnitType),
                    _ => Ok(Token::Ident(tmp)),
                }
            }

            c if c.is_numeric() => {
                let mut tmp = String::new();
                tmp.push(c);
                self.next_char();
                let tmp = format!("{}{}", tmp, self.eat_num_dump());
                Ok(Token::Num(tmp))
            }

            '(' => {
                self.next_char();
                Ok(Token::LParen)
            }

            ')' => {
                self.next_char();
                Ok(Token::RParen)
            }

            '{' => {
                self.next_char();
                Ok(Token::RBrace)
            }

            '}' => {
                self.next_char();
                Ok(Token::LBrace)
            }

            ',' => {
                self.next_char();
                Ok(Token::Comma)
            }

            ':' => {
                self.next_char();
                Ok(Token::Colon)
            }

            ';' => {
                self.next_char();
                Ok(Token::SemiColon)
            }

            '\0' => {
                self.next_char();
                Ok(Token::EOF)
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
                Ok(t) => match t {
                    Token::EOF => break,
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
