use derive_more::Constructor;
use std::{collections::{BTreeMap, HashMap}, fmt::Debug, hash::Hash, ops::Range, sync::atomic::AtomicUsize};
use lazy_static::lazy_static;

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref var_counter: AtomicUsize = AtomicUsize::new(0);
}

// Ids used to index data in context.

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldId(usize);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarId(usize);

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
        field: FieldId,
        value: Box<Value>,
    },
    Struct {
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
    },
    Ref(TypeId)
}

#[derive(Debug, Clone)]
pub enum Builtin {
    AddI(VarId, VarId)
}

#[derive(Clone)]
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

pub enum Decl {
    /// Using a TypeId allows recursive types. It also allows differentiation between
    /// similar sum and product types.
    Typedef(TypeId, Type),
    Expr(Expr)
}

impl VarId {
    pub fn new() -> Self {
        VarId(var_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

impl Debug for VarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("V{}", self.0))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::Double(l0), Self::Double(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Function { .. }, Self::Function { .. }) => false, // Cannot compare functions.
            (Self::Enum { field: l_field, value: l_value }, Self::Enum { field: r_field, value: r_value }) => l_field == r_field && l_value == r_value,
            (Self::Struct { fields: l_fields }, Self::Struct { fields: r_fields }) => l_fields == r_fields,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(v, _) => {
                match v {
                    Value::Unit => f.write_str("():()"),
                    Value::Integer(v) => f.write_fmt(format_args!("{}:Integer", v)),
                    Value::Double(v) => f.write_fmt(format_args!("{}:Double", v)),
                    Value::String(v) => f.write_fmt(format_args!("{}:String", v)),
                    Value::Function { params, body } => {
                        let mut args = f.debug_tuple("");
                        for (v,t) in params {
                            args.field(&format!("{:?}:{:?}", v, t));
                        }
                        args.finish()?;
                        f.write_str(" => ")?;
                        body.fmt(f)
                    },
                    Value::Enum { field, value } => todo!(),
                    Value::Struct { fields } => todo!(),
                }
            },
            Self::Var(v) => f.write_fmt(format_args!("V{}", v.0)),
            Self::Let { bind, expr, .. } => {
                f.write_str("let {")?;
                let mut first = true;
                for (v, e) in bind {
                    if first {
                        first = false
                    } else {
                        f.write_str(",")?;
                    }
                    f.write_fmt(format_args!("{:?}={:?}", v, e))?;
                }
                f.write_str("}")?;
                f.write_fmt(format_args!(" in {:?}", expr))
            },
            Self::Call { func, args, .. } => {
                f.write_fmt(format_args!("{:?}", func))?;
                let mut first = true;
                for arg in args {
                    if first {
                        f.write_str("(")?;
                        first = false
                    } else {
                        f.write_str(",")?;
                    }
                    f.write_fmt(format_args!("{:?}", arg))?;
                }
                f.write_str(")")
            },
            Self::Builtin(arg0) => f.debug_tuple("Builtin").field(arg0).finish(),
        }
    }
}

impl Expr {
    pub fn unbound_variables(&self) -> Vec<VarId> {
        match &self {
            Expr::Value(value, debug_info) => Vec::new(),
            Expr::Var(var_id) => Vec::from([*var_id]),
            Expr::Let { bind, expr, debug } => {
                let mut vars = expr.unbound_variables();
                vars.retain(|it| !bind.contains_key(it));
                vars
            },
            Expr::Call { func, args, debug } => {
                let mut vars = vec![*func];
                for arg in args {
                    vars.extend(arg.unbound_variables());
                }
                vars
            },
            Expr::Builtin(Builtin::AddI(v1, v2)) => Vec::from([*v1,*v2]),
        }
    }

    /**
     * This replaces every instance of the variables in the keys with the mapped vars.
     * This can change the semantics of the expression.
     */
    fn with_mapped_vars(&self, mapping: &HashMap<VarId, VarId>) -> Expr {
        let map = |v| mapping.get(v).cloned().unwrap_or(*v);

        match &self {
            Expr::Value(value, debug_info) => {
                match value {
                    Value::Unit => self.clone(),
                    Value::Integer(_) => self.clone(),
                    Value::Double(_) => self.clone(),
                    Value::String(_) => self.clone(),
                    Value::Function { params, body } => {
                        let params = params
                            .iter()
                            .map(|(v, t)| (map(v), t.clone()))
                            .collect();
                        Expr::Value(Value::Function { params, body: Box::new(body.with_mapped_vars(mapping)) }, None)
                    },
                    Value::Enum { field, value } => todo!(),
                    Value::Struct { fields } => todo!(),
                }
            },
            Expr::Var(var_id) => Expr::Var(map(var_id)),
            Expr::Let { bind, expr, debug } => {
                let bind = HashMap::from_iter(bind.iter()
                    .map(|(v, e)| (map(v), e.with_mapped_vars(mapping)))
                );
                let expr = Box::new(expr.with_mapped_vars(mapping));
                Expr::Let { bind, expr, debug: debug.clone() }
            },
            Expr::Call { func, args, debug } => {
                let func = map(func);
                let args = args.iter()
                    .map(|it| it.with_mapped_vars(mapping))
                    .collect();

                Expr::Call { func, args, debug: debug.clone() }
            },
            Expr::Builtin(builtin) => {
                Expr::Builtin(match builtin {
                    Builtin::AddI(v0, v1) => Builtin::AddI(map(v0), map(v1)),
                })
            },
        }
    }

    /**
     * Replace the only the bound vars in this expression with new bound vars if it
     * already exists according to the provided function.
     * 
     * This should not change the semantics of the expression.
     */
    pub fn with_fresh_vars(&self, does_var_exist: &dyn Fn(&VarId) -> bool) -> Expr {
        match &self {
            Expr::Value(v, d) => {
                match v {
                    Value::Unit => self.clone(),
                    Value::Integer(_) => self.clone(),
                    Value::Double(_) => self.clone(),
                    Value::String(_) => self.clone(),
                    Value::Function { params, body } => {
                        let mappings = HashMap::from_iter(
                            params
                                .iter()
                                .map(|(v , _)| v)
                                .filter(|it| does_var_exist(it))
                                .map(|it| (*it, VarId::new()))
                        );

                        let params = params.iter()
                            .map(|(v, t)| (mappings.get(v).cloned().unwrap_or(*v), t.clone()))
                            .collect();

                        let body = Box::new(body.with_mapped_vars(&mappings));

                        Expr::Value(Value::Function { params, body }, d.clone())
                    },
                    Value::Enum { field, value } => todo!(),
                    Value::Struct { fields } => todo!(),
                }
            },
            Expr::Var(_) => self.clone(), // Notice, this var is unbound within itself so we do not replace it.
            Expr::Let { bind, expr, debug } => {
                let mappings = HashMap::from_iter(
                    bind
                        .keys()
                        .filter(|it| does_var_exist(it))
                        .map(|it| (*it, VarId::new()))
                );

                let bind = HashMap::from_iter(
                    bind.iter()
                        .map(|(v, e)| (mappings.get(v).cloned().unwrap_or(*v), e.clone()))
                );
                let expr = expr.with_mapped_vars(&mappings);

                Expr::Let { bind, expr: Box::new(expr), debug: debug.clone() }
            },
            Expr::Call { func, args, debug } => {
                let args = args.iter()
                    .map(|e| e.with_fresh_vars(&does_var_exist))
                    .collect();

                Expr::Call { func: *func, args, debug: debug.clone() }
            },
            Expr::Builtin(_) => self.clone(),
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