use crate::type_def::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // symbols
    Func,
    FuncAnon,
    RParen,
    LParen,
    RBrace,
    LBrace,
    Comma,
    Colon,
    SemiColon,
    Arrow,

    // premitive values
    Num(String),
    Ident(String),
    UnitVal,

    // types
    // for function type we will use syntax like (Type,...)->Type
    // so we don't need any token for function type.
    I32,
    UnitType,
    FuncType,

    // EOF
    EOF,
}

#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    I32 {
        val: i32,
    },
    NamedFunc {
        name: String,
        args_def: Vec<ArgDecl>,
        ret_decl: Type,
        block: Box<Expr>,
    },
    Unit,
    AnonFunc {
        args_decl: Vec<ArgDecl>,
        ret_decl: Type,
        block: Box<Expr>,
    },

    // Block
    Block {
        exprs: Vec<Box<Expr>>,
    },

    // Variable
    Var {
        name: String,
    },

    // Function app
    FuncApp {
        callee: Box<Expr>,
        args: Vec<Box<Expr>>,
    },
}

#[derive(Debug, Clone)]
pub struct ArgDecl {
    pub vname: String,
    pub vtype: Type,
}

impl ArgDecl {
    pub fn into_type(self) -> Type {
        self.vtype
    }
}
