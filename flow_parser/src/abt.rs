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
pub enum Builtin {
    AddI(VarId, VarId)
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

    Builtin(Builtin)

    // Todo:
    //
    // Extern {
    //     lang: ExternLang,
    //     def: FFI(...)
    // }
    //
    // 
}

impl Expr {
    pub fn unbound_variables(&self) -> Vec<VarId> {
        match &self {
            Expr::Value(value, debug_info) => Vec::new(),
            Expr::Var(var_id) => Vec::from([*var_id]),
            Expr::Let { bind, expr, debug } => todo!(),
            Expr::Call { func, args, debug } => todo!(),
            Expr::Builtin(Builtin::AddI(v1, v2)) => Vec::from([*v1,*v2]),
        }
    }
}

impl Type {
    #[allow(non_snake_case)]
    fn Function(params: &[Type], ret: Type) -> Type {
        Type::Function { params: Vec::from(params), ret: Box::new(ret.clone()) }
    }
}

impl Builtin {
    pub fn get_type(&self) -> Type {
        match &self {
            Builtin::AddI(_, _) => Type::Function(&[Type::Integer, Type::Integer], Type::Integer),
        }
    }
}