use derive_more::Constructor;
use std::{collections::HashMap, ops::Range};

// Ids used to index data in context.

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarId(pub usize);


// Are these useful for closures/lambda implementation?
// Represent a lambda as an expression with captures as bound variables?

pub struct Abstractor {
    vars: Vec<VarId>,
    expr: Expr
}

pub struct Substitution {
    bind: Vec<Expr>,
    expr: Abstractor
}



#[derive(Debug, Clone, Constructor)]
pub struct DebugInfo {
    file: FileId,
    span: Range<usize>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Integer(i64),
    Double(f64),
    String(String),
    Function {
        params: Vec<VarId>,
        body: Box<Expr>,
    },
    Enum {
        kind: TypeId,
        value: Box<Value>,
    },
    Struct {
        kind: TypeId,
        fields: HashMap<FieldId, Box<Value>>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value, Option<DebugInfo>),
    Var(VarId),

    Let {
        bind: HashMap<VarId, Expr>,
        expr: Box<Expr>,
        debug: Option<DebugInfo>
    },

    Call {
        func: VarId,
        args: Vec<Expr>,
        debug: Option<DebugInfo>
    },
}
