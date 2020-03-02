use crate::syntax::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // premitive types
    I32,
    Unit,

    // function type
    Func {
        args: Vec<Box<Type>>,
        ret: Box<Type>,
    },

    // user defined typ
    UserType {
        name: String,
    },
}

pub struct TypedExpr {
    pub expr: Box<Expr>,
    pub expr_type: Option<Type>,
}

impl TypedExpr {
    pub fn new(expr: Box<Expr>, expr_type: Option<Type>) -> TypedExpr {
        TypedExpr { expr, expr_type }
    }
}
