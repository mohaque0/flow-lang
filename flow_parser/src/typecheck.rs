use std::collections::HashMap;

use crate::abt::{Expr, Type, TypeId, VarId, TYPE_ID_DOUBLE, TYPE_ID_INT, TYPE_ID_STRING, TYPE_ID_UNIT};

#[derive(Clone)]
pub struct TypecheckContext {
    known_types: HashMap<VarId, Type>
}

impl TypecheckContext {
    pub fn new() -> Self {
        let known_types = HashMap::new();
        Self {
            known_types
        }
    }

    pub fn get_var_type(&self, var: &VarId) -> Option<Type> {
        return self.known_types.get(var).cloned();
    }
}

pub fn typecheck(ctx: &TypecheckContext, expr: &Expr) -> Option<Type> {
    match expr {
        Expr::Value(value, _) => {
            match value {
                crate::abt::Value::Unit => Some(Type::Unit),
                crate::abt::Value::Integer(_) => Some(Type::Integer),
                crate::abt::Value::Double(_) => Some(Type::Double),
                crate::abt::Value::String(_) => Some(Type::String),
                crate::abt::Value::Function { params, body } => {
                    let mut subctx = ctx.clone();
                    subctx.known_types.extend(params.iter().cloned());
                    typecheck(&subctx, body)
                },
                crate::abt::Value::Enum { kind, field, value } => todo!(),
                crate::abt::Value::Struct { kind, fields } => todo!(),
            }
        },
        Expr::Var(var_id) => ctx.get_var_type(var_id),
        Expr::Let { bind, expr, .. } => {
            let mut subctx = ctx.clone();
            bind.iter().for_each(|(v, e)| {
                subctx.known_types.insert(*v, typecheck(&subctx, &e).expect("Unknown type."));
            });
            typecheck(&subctx, expr)
        },
        Expr::Call { func, args, .. } => {
            let ft = ctx.get_var_type(func).expect("Unknown function type.");
            if let Type::Function { params, ret } = ft {
                return Some(*ret);
            } else {
                panic!("TypeMismatch: {:?} is not a function.", ft);
            }
        },
    }
}