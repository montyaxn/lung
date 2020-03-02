use std::collections::HashMap;

use crate::parser::*;
use crate::syntax::*;
use crate::type_def::*;

#[cfg(test)]
mod test_typing {
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
            Token::UnitVal,
            Token::RBrace,
        ]);
        let expr = *parser.parse_program().unwrap();
        expr.into_typed_expr(&mut Context::new()).unwrap();
    }
}

struct VarTypeTable{
    table : HashMap<String,Type>,
}

impl VarTypeTable{
    fn get(&self,name : &String) -> Option<&Type>{
        self.table.get(name)
    }
    pub fn from_args_decl(decls: Vec<ArgDecl>) -> VarTypeTable {
        let mut ret = VarTypeTable {
            table: HashMap::new()
        };
        for d in decls {
            ret.table.insert(d.vname, d.vtype);
        }
        ret
    }
}

struct Context {
    layered_table : Vec<VarTypeTable>,
    searching_depth : usize,
    visible_bottom: usize,
    visible_top: usize,
}

impl Context{
    fn get(&mut self,name : String) -> Result<Type,String>{
        while self.visible_bottom <= self.searching_depth && self.searching_depth <= self.visible_top {
            match self.layered_table[self.searching_depth].get(&name) {
                Some(t) => {
                    return Ok(t.clone());
                }
                None => {}
            }
            self.searching_depth-=1
        }
        Err(String::from("Could not found variables"))
    }

    pub fn push_table_from_argsdecl(&mut self, args_decl: Vec<ArgDecl>) {
        self.layered_table.push(VarTypeTable::from_args_decl(args_decl));
        self.visible_top += 1;
    }

    pub fn new() -> Context {
        Context {
            layered_table: Vec::new(),
            searching_depth: 0,
            visible_bottom: 0,
            visible_top: 0,
        }
    }
}

impl Expr{
    fn into_typed_expr(self, cxt: &mut Context) -> Result<TypedExpr, String> {
        match &self {
            Expr::Unit => {
                Ok(TypedExpr::new(Box::from(self),Some(Type::Unit)))
            }
            Expr::I32{val:_} => {
                Ok(TypedExpr::new(Box::from(self), Some(Type::I32)))
            }
            Expr::Var{name} => {
                let expr_type = cxt.get(name.clone())?;
                Ok(TypedExpr::new(Box::from(self), Some(expr_type)))
            }
            Expr::Block { exprs } => {
                let mut last = TypedExpr::new(Box::from(Expr::Unit), Some(Type::Unit));
                for expr in exprs.clone() {
                    match expr.into_typed_expr(cxt) {
                        Ok(s) => { last = s }
                        Err(e) => return Err(e)
                    }
                }
                Ok(last)
            }
            Expr::AnonFunc { args_decl, ret_decl, block } => {
                cxt.push_table_from_argsdecl(args_decl.clone());
                let typed_block = block.clone().into_typed_expr(cxt)?;
                match typed_block.expr_type {
                    Some(ref t) if *t == *ret_decl => (),
                    _ => return Err(String::from("Expected but found"))
                }


                let args = args_decl.iter().map(|x| Box::from(x.clone().vtype)).collect();
                let ret = Box::from(ret_decl.clone());
                let expr = Box::from(self);
                let expr_type = Some(Type::Func { args, ret });
                Ok(TypedExpr { expr, expr_type })
            }
            Expr::FuncApp { callee, args } => {
                // calleeの型を調べる
                let callee_type = (*callee.clone()).into_typed_expr(cxt)?;
                let fn_args_ty;
                let ret_ty;
                match callee_type.expr_type {
                    Some(Type::Func { args, ret }) => {
                        fn_args_ty = args;
                        ret_ty = ret;
                    }
                    _ => {
                        return Err(String::from("Error: Callee must have function type"))
                    }
                }

                // argsの型を調べる
                let mut app_args_type = Vec::new();
                for e in args.clone() {
                    let e_to_typedexpr = (*e).into_typed_expr(cxt)?;
                    let e_type = match e_to_typedexpr.expr_type {
                        Some(t) => t,
                        None => return Err(String::from("型推論実装してねえ"))
                    };
                    app_args_type.push(e_type);
                }

                // calleeのargsの型とargsの型が一致するか調べる
                let mut fn_args_type = fn_args_ty.iter();
                let mut app_args_type = app_args_type.iter();
                loop {
                    match fn_args_type.next() {
                        Some(tf) => {
                            match app_args_type.next() {
                                Some(ta) => {
                                    if !(*ta == **tf) {
                                        return Err(String::from("Expected but"))
                                    }
                                }
                                None => {
                                    return Err(String::from("The number of the args is expected to be {}"))
                                }
                            }
                        }
                        None => {
                            match app_args_type.next() {
                                None => {
                                    break
                                }
                                Some(_) => {
                                    return Err(String::from("The number of the args is expected to be {}"))
                                }
                            }
                        }
                    }
                }
                Ok(TypedExpr { expr: Box::from(self), expr_type: Some(*ret_ty) })
            }
            _ => {
                Err(String::from("hello"))
            }
        }
    }
}