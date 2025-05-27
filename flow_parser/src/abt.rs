use derive_more::Constructor;
use std::{collections::{BTreeMap, HashMap}, hash::Hash, ops::Range};

// Ids used to index data in context.

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarId(pub usize);


pub const TYPE_ID_UNIT: TypeId = TypeId(0);
pub const TYPE_ID_INT: TypeId = TypeId(1);
pub const TYPE_ID_DOUBLE: TypeId = TypeId(2);
pub const TYPE_ID_STRING: TypeId = TypeId(3);



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
        params: Vec<(VarId, Type)>,
        body: Box<Expr>,
    },
    Enum {
        kind: TypeId,
        field: FieldId,
        value: Box<Value>,
    },
    Struct {
        kind: TypeId,
        fields: HashMap<FieldId, Box<Value>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Unit,
    Integer,
    Double,
    String,
    Function {
        params: Vec<Type>,
        ret: Box<Type>
    },
    Enum {
        kinds: BTreeMap<FieldId, Type>
    },
    Struct {
        fields: BTreeMap<FieldId, Type>
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