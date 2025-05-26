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
pub enum Type {
    Unit,
    Integer,
    Double,
    String,
    Function {
        params: Vec<TypeId>,
        ret: TypeId
    },
    Enum {
        kinds: HashMap<FieldId, TypeId>
    },
    Struct {
        fields: HashMap<FieldId, TypeId>
    }
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

    // Todo:
    //
    // Extern {
    //     lang: ExternLang,
    //     def: FFI(...)
    // }
    //
    // BuiltIn {}
}

impl Expr {
    pub fn unbound_variables(&self) -> Vec<VarId> {
        match &self {
            Expr::Value(value, debug_info) => Vec::new(),
            Expr::Var(var_id) => Vec::from([*var_id]),
            Expr::Let { bind, expr, debug } => todo!(),
            Expr::Call { func, args, debug } => todo!(),
        }
    }
}