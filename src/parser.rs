use crate::lexer::*;
use crate::syntax::*;
use crate::type_def::*;

#[cfg(test)]
mod parser_test {
    use super::*;

    #[test]
    fn test() {
        let mut lexer = Lexer::from_file("src/test/test_parser.txt").unwrap();
        let mut parser = Parser::new(lexer.lex().unwrap());
        parser.parse_program().unwrap();
    }
}

pub struct Parser {
    tokens: std::vec::IntoIter<Token>,
    ctk: TokenKind,
    cti: TokenInfo,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into_iter(),
            ctk: TokenKind::EOF,
            cti: TokenInfo {
                s_col: 0,
                s_row: 0,
                e_col: 0,
                e_row: 0,
            },
        }
    }

    fn make_error(&mut self, expectation: &str) -> String {
        // Lung parser use form of Error like the following
        // Error at [s_row:s_col-e_row:e_col] : Expected ~~
        let msg = format!(
            "Error at {} : Expected {}",
            self.cti.to_string(),
            expectation
        );
        String::from(msg)
    }

    pub fn parse_program(&mut self) -> Result<Box<Expr>, String> {
        self.next_token();
        self.read_expr()
    }

    fn next_token(&mut self) {
        match self.tokens.next() {
            Some(Token {
                kind: TokenKind::EOF,
                info: _,
            }) => {
                self.next_token();
            }
            Some(t) => {
                self.ctk = t.kind;
                self.cti = t.info;
            }
            None => {
                self.ctk = TokenKind::EOF;
            }
        }
    }

    fn ct_check(&mut self, token: TokenKind) -> bool {
        match &self.ctk {
            TokenKind::Num(_) => match token {
                TokenKind::Num(_) => true,
                _ => false,
            },
            TokenKind::Ident(_) => match token {
                TokenKind::Ident(_) => true,
                _ => false,
            },
            t => t == &token,
        }
    }

    fn read_args(&mut self) -> Result<Vec<Box<Expr>>, String> {
        let mut tmp = Vec::new();
        if self.ctk == TokenKind::LParen {
            self.next_token();
            return Ok(tmp);
        }
        loop {
            if !Parser::lead_expr(self.ctk.clone()) {
                return Err(self.make_error("EXPR"));
            }
            tmp.push(self.read_expr()?);
            match self.ctk {
                TokenKind::RParen => break,
                TokenKind::Comma => {
                    self.next_token();
                }
                _ => {
                    return Err(self.make_error("[RPAREN,COMMA]"));
                }
            }
        }
        self.next_token();
        Ok(tmp)
    }

    fn read_block(&mut self) -> Result<Box<Expr>, String> {
        let mut exprs = vec![self.read_expr()?];

        loop {
            match self.ctk {
                TokenKind::SemiColon => {
                    self.next_token();
                    exprs.push(match self.read_expr() {
                        Ok(e) => e,
                        error => {
                            return error;
                        }
                    });
                }

                TokenKind::RBrace => {
                    self.next_token();
                    break;
                }
                _ => return Err(self.make_error("[SEMICOLON,RBRACE]")),
            }
        }
        Ok(Box::from(Expr::Block { exprs }))
    }

    fn read_args_decl(&mut self) -> Result<Vec<ArgDecl>, String> {
        let mut args_def = Vec::new();
        loop {
            let vname = match self.ctk.clone() {
                TokenKind::Ident(s) => {
                    self.next_token();
                    s
                }
                _ => {
                    return Err(self.make_error("IDENT"));
                }
            };
            match self.ctk {
                TokenKind::Colon => self.next_token(),
                _ => return Err(self.make_error("COLON")),
            }
            let vtype = self.read_type()?;
            args_def.push(ArgDecl { vname, vtype });
            match self.ctk {
                TokenKind::RParen => {
                    self.next_token();
                    break;
                }
                TokenKind::Comma => {
                    self.next_token();
                }
                _ => {
                    return Err(self.make_error("[RPAREN,COMMA]"));
                }
            }
        }
        Ok(args_def)
    }

    fn read_type_args(&mut self) -> Result<Vec<Box<Type>>, String> {
        let mut args = Vec::new();
        loop {
            let tmp = Box::from(self.read_type()?);
            args.push(tmp);
            match self.ctk {
                TokenKind::Comma => self.next_token(),
                TokenKind::RParen => break,
                _ => return Err(self.make_error("[COMMA,RPAREN]")),
            }
        }
        self.next_token();
        Ok(args)
    }

    fn read_type(&mut self) -> Result<Type, String> {
        let ret;
        ret = match self.ctk.clone() {
            TokenKind::Ident(name) => {
                self.next_token();
                Type::UserType { name }
            }
            TokenKind::I32 => {
                self.next_token();
                Type::I32
            }
            TokenKind::UnitType => {
                self.next_token();
                Type::Unit
            }
            TokenKind::FuncType => {
                if !self.ct_check(TokenKind::LParen) {
                    return Err(self.make_error("LPAREN"));
                }
                let args = self.read_type_args()?;
                let ret = Box::from(self.read_type()?);
                Type::Func { args, ret }
            }
            _ => return Err(self.make_error("TYPE")),
        };
        Ok(ret)
    }

    fn read_ret_decl(&mut self) -> Result<Type, String> {
        match self.ctk {
            TokenKind::Arrow => self.next_token(),
            _ => return Err(self.make_error("ARROW")),
        }
        Ok(self.read_type()?)
    }

    fn read_anon_func(&mut self) -> Result<Box<Expr>, String> {
        match self.ctk {
            TokenKind::LParen => {
                self.next_token();
            }
            _ => {
                return Err(self.make_error("LPAREN"));
            }
        }
        let args_decl = self.read_args_decl()?;
        let ret_decl = self.read_ret_decl()?;
        let block = match self.ctk {
            TokenKind::LBrace => {
                self.next_token();
                self.read_block()?
            }
            _ => return Err(self.make_error("BLOCK")),
        };
        Ok(Box::from(Expr::AnonFunc {
            args_decl,
            ret_decl,
            block,
        }))
    }

    fn lead_expr(token: TokenKind) -> bool {
        match token {
            ref t if Parser::lead_simple_expr(t.clone()) => true,
            _ => false,
        }
    }

    fn read_expr(&mut self) -> Result<Box<Expr>, String> {
        let mut ret_expr: Box<Expr>;
        match self.ctk.clone() {
            ref t if Parser::lead_simple_expr(t.clone()) => {
                ret_expr = self.read_simple_expr()?;
            }
            _ => return Err(self.make_error("EXPR")),
        }

        loop {
            match self.ctk {
                TokenKind::LParen => {
                    self.next_token();
                    let args = self.read_args()?;
                    ret_expr = Box::from(Expr::FuncApp {
                        callee: ret_expr,
                        args: args,
                    })
                }

                _ => {
                    break;
                }
            }
        }
        Ok(ret_expr)
    }

    fn lead_simple_expr(token: TokenKind) -> bool {
        match &token {
            TokenKind::Num(_) => true,
            TokenKind::Ident(_) => true,
            TokenKind::Func => true,
            TokenKind::FuncAnon => true,
            TokenKind::LParen => true,
            TokenKind::RBrace => true,
            TokenKind::UnitVal => true,
            _ => false,
        }
    }

    fn read_simple_expr(&mut self) -> Result<Box<Expr>, String> {
        let ct = self.ctk.clone();
        let ret_expr;
        match ct {
            TokenKind::Num(s) => {
                self.next_token();
                ret_expr = Box::from(Expr::I32 {
                    val: s.parse().unwrap(),
                });
            }

            TokenKind::Ident(s) => {
                self.next_token();
                ret_expr = Box::from(Expr::Var { name: s.clone() })
            }

            TokenKind::UnitVal => {
                self.next_token();
                ret_expr = Box::from(Expr::Unit)
            }

            TokenKind::FuncAnon => {
                self.next_token();
                ret_expr = self.read_anon_func()?;
            }

            TokenKind::LParen => {
                ret_expr = self.read_expr()?;
                if !self.ct_check(TokenKind::RParen) {
                    return Err(self.make_error("RPAREN"));
                }
                self.next_token();
            }

            TokenKind::LBrace => {
                self.next_token();

                ret_expr = self.read_block()?;
            }

            _ => return Err(self.make_error("EXPR")),
        }

        Ok(ret_expr)
    }
}
