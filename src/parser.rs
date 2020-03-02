use crate::lexer::*;
use crate::syntax::*;
use crate::type_def::*;

#[cfg(test)]
mod parser_test {
    use super::*;

    #[test]
    fn test() {
        let mut parser = Parser::new(vec![
            Token::FuncAnon,
            Token::LParen,
            Token::Ident(String::from("hello")),
            Token::Colon,
            Token::UnitType,
            Token::Comma,
            Token::Ident(String::from("hello")),
            Token::Colon,
            Token::I32,
            Token::RParen,
            Token::Arrow,
            Token::UnitType,
            Token::LBrace,
            Token::Num(String::from("123")),
            Token::SemiColon,
            Token::Num(String::from("123")),
            Token::RBrace,
        ]);
        println!("{:?}", parser.parse_program().unwrap());
    }
}

pub struct Parser {
    tokens: std::vec::IntoIter<Token>,
    ct: Token,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into_iter(),
            ct: Token::EOF,
        }
    }

    pub fn parse_program(&mut self) -> Result<Box<Expr>, String> {
        self.next_token();
        self.read_expr()
    }

    fn next_token(&mut self) {
        match self.tokens.next() {
            Some(Token::EOF) => {
                self.next_token();
            }
            Some(t) => {
                self.ct = t;
            }
            None => {
                self.ct = Token::EOF;
            }
        }
    }

    fn ct_check(&mut self, token: Token) -> bool {
        match &self.ct {
            Token::Num(_) => match token {
                Token::Num(_) => true,
                _ => false,
            },
            Token::Ident(_) => match token {
                Token::Ident(_) => true,
                _ => false,
            },
            t => t == &token,
        }
    }

    fn read_args(&mut self) -> Result<Vec<Box<Expr>>, String> {
        let mut tmp = Vec::new();
        if self.ct == Token::LParen {
            self.next_token();
            return Ok(tmp);
        }
        loop {
            if !Parser::lead_expr(self.ct.clone()) {
                return Err(String::from("Error: Expected {[EXPR]}"));
            }
            tmp.push(self.read_expr()?);
            match self.ct {
                Token::RParen => break,
                Token::Comma => {
                    self.next_token();
                }
                _ => {
                    return Err(String::from("Error: Expected {[RPAREN],[COMMA]}"));
                }
            }
        }
        self.next_token();
        Ok(tmp)
    }

    fn read_block(&mut self) -> Result<Box<Expr>, String> {
        let mut exprs = vec![self.read_expr()?];

        loop {
            match self.ct {
                Token::SemiColon => {
                    self.next_token();
                    exprs.push(match self.read_expr() {
                        Ok(e) => e,
                        error => {
                            return error;
                        }
                    });
                }

                Token::RBrace => {
                    self.next_token();
                    break;
                }
                _ => return Err(String::from("Error: Expected {';','}'}")),
            }
        }
        Ok(Box::from(Expr::Block { exprs }))
    }

    fn read_args_decl(&mut self) -> Result<Vec<ArgDecl>, String> {
        let mut args_def = Vec::new();
        loop {
            let vname = match self.ct.clone() {
                Token::Ident(s) => {
                    self.next_token();
                    s
                }
                _ => {
                    return Err(String::from("Expected {[Ident]}"));
                }
            };
            match self.ct {
                Token::Colon => self.next_token(),
                _ => return Err(String::from("Expected {[Colon]}")),
            }
            let vtype = match self.ct.clone() {
                Token::Ident(s) => Type::UserType { name: s },
                Token::I32 => Type::I32,
                Token::UnitType => Type::Unit,
                _ => return Err(String::from("Expected {[Ident],[Type]}")),
            };
            self.next_token();
            args_def.push(ArgDecl { vname, vtype });
            match self.ct {
                Token::RParen => {
                    self.next_token();
                    break;
                }
                Token::Comma => {
                    self.next_token();
                }
                _ => {
                    return Err(String::from("Expected {')',','}"));
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
            match self.ct {
                Token::Comma => self.next_token(),
                Token::RParen => break,
                _ => return Err(String::from("Expected {[Colon],[RParen]}"))
            }
        }
        self.next_token();
        Ok(args)
    }

    fn read_type(&mut self) -> Result<Type, String> {
        let ret;
        ret = match self.ct.clone() {
            Token::Ident(name) => {
                self.next_token();
                Type::UserType { name }
            }
            Token::I32 => {
                self.next_token();
                Type::I32
            }
            Token::UnitType => {
                self.next_token();
                Type::Unit
            }
            Token::FuncType => {
                if !self.ct_check(Token::LParen) {
                    return Err(String::from("Expected {[LParen]}"))
                }
                let args = self.read_type_args()?;
                let ret = Box::from(self.read_type()?);
                Type::Func { args, ret }
            }
            _ => { return Err(String::from("Expected {[Type]}")) }
        };
        Ok(ret)
    }

    fn read_ret_decl(&mut self) -> Result<Type, String> {
        match self.ct {
            Token::Arrow => { self.next_token() },
            _ => return Err(String::from("Expected {[Arrow]}"))
        }
        Ok(self.read_type()?)
    }

    fn read_anon_func(&mut self) -> Result<Box<Expr>, String> {
        match self.ct {
            Token::LParen => {
                self.next_token();
            }
            _ => {
                return Err(String::from("Expected {[LParen]}"));
            }
        }
        let args_decl = self.read_args_decl()?;
        let ret_decl = self.read_ret_decl()?;
        let block = match self.ct {
            Token::LBrace => {
                self.next_token();
                self.read_block()?
            }
            _ => return Err(String::from("Expected {[Block]}")),
        };
        Ok(Box::from(Expr::AnonFunc { args_decl, ret_decl, block }))
    }

    fn lead_expr(token: Token) -> bool {
        match token {
            ref t if Parser::lead_simple_expr(t.clone()) => true,
            _ => false,
        }
    }

    fn read_expr(&mut self) -> Result<Box<Expr>, String> {
        let mut ret_expr: Box<Expr>;
        match self.ct.clone() {
            ref t if Parser::lead_simple_expr(t.clone()) => {
                ret_expr = self.read_simpl_expr()?;
            }
            _ => return Err(String::from("Error: Expected {[EXPR]}")),
        }

        loop {
            match self.ct {
                Token::LParen => {
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

    fn lead_simple_expr(token: Token) -> bool {
        match &token {
            Token::Num(_) => true,
            Token::Ident(_) => true,
            Token::Func => true,
            Token::FuncAnon => true,
            Token::LParen => true,
            Token::RBrace => true,
            Token::UnitVal => true,
            _ => false,
        }
    }

    fn read_simpl_expr(&mut self) -> Result<Box<Expr>, String> {
        let ct = self.ct.clone();
        let ret_expr;
        match ct {
            Token::Num(s) => {
                self.next_token();
                ret_expr = Box::from(Expr::I32 {
                    val: s.parse().unwrap(),
                });
            }

            Token::Ident(s) => {
                self.next_token();
                ret_expr = Box::from(Expr::Var { name: s.clone() })
            }

            Token::UnitVal => {
                self.next_token();
                ret_expr = Box::from(Expr::Unit)
            }

            Token::FuncAnon => {
                self.next_token();
                ret_expr = self.read_anon_func()?;
            }

            Token::LParen => {
                ret_expr = self.read_expr()?;
                if !self.ct_check(Token::RParen) {
                    return Err(String::from("Error: Expected {[RPAREN]}"));
                }
                self.next_token();
            }

            Token::LBrace => {
                self.next_token();

                ret_expr = self.read_block()?;
            }

            _ => return Err(String::from("Expected {[NUM],[IDENT]}")),
        }

        Ok(ret_expr)
    }
}
