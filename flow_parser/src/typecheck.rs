use std::collections::HashMap;

use crate::abt::{Builtin, Expr, FieldId, Type, TypeId, VarId};

#[derive(Clone)]
pub struct TypecheckContext {
    type_references: HashMap<TypeId, Type>,
    known_var_types: HashMap<VarId, Type>,
    field_owners: HashMap<FieldId, TypeId>
}

impl TypecheckContext {
    pub fn new() -> Self {
        Self {
            type_references: HashMap::new(),
            known_var_types: HashMap::new(),
            field_owners: HashMap::new()
        }
    }

    pub fn add_type(&mut self, id: TypeId, t: Type) {
        match &t {
            Type::Enum { kinds } => {
                for f in kinds.keys() {
                    self.field_owners.insert(*f, id);
                }
            },
            Type::Struct { fields } => {
                for f in fields.keys() {
                    self.field_owners.insert(*f, id);
                }
            },
            _ => {}
        }
        self.type_references.insert(id, t);
    }

    pub fn add_var(&mut self, var_id: VarId, var_type: Type) {
        self.known_var_types.insert(var_id, var_type);
    }

    pub fn get_var_type(&self, var: &VarId) -> Option<Type> {
        let t = self.known_var_types.get(var);
        match t {
            Some(Type::Ref(tid)) => self.resolve_type(tid),
            _ => t.cloned()
        }
    }

    pub fn get_field_owner(&self, field: &FieldId) -> Option<TypeId> {
        self.field_owners.get(field).cloned()
    }

    pub fn resolve_type(&self, t: &TypeId) -> Option<Type> {
        return self.type_references.get(t).cloned();
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
                    params.iter().for_each(|(v, t)| subctx.add_var(*v, t.clone()));
                    let ret = typecheck(&subctx, body)?;

                    Some(Type::Function {
                        params: params.iter().map(|(_, it)| it.clone()).collect(),
                        ret: Box::new(ret)
                    })
                },
                crate::abt::Value::Enum { field, .. } => {
                    let tid = ctx.get_field_owner(field);
                    match tid {
                        Some(tid) => ctx.resolve_type(&tid),
                        None => None,
                    }
                },
                crate::abt::Value::Struct { fields } => {
                    let owners: Vec<Option<TypeId>> = fields.iter().map(|(field, _)| ctx.get_field_owner(field)).collect();

                    // Check that all fields are of the same type.
                    if owners.len() > 0 {
                        let first = owners[0].clone();
                        for owner in &owners {
                            if owner != &first {
                                return None;
                            }
                        }

                        if let Some(tid) = first {
                            return ctx.resolve_type(&tid);
                        }
                    }

                    return None;
                },
            }
        },
        Expr::Var(var_id, _) => ctx.get_var_type(var_id),
        Expr::Let { bind, expr, .. } => {
            let mut subctx = ctx.clone();
            bind.iter().for_each(|(v, e)| {
                subctx.known_var_types.insert(*v, typecheck(&subctx, &e).expect("Unknown type."));
            });
            typecheck(&subctx, expr)
        },
        Expr::Call { func, args, .. } => {
            let ft = ctx.get_var_type(func).expect("Unknown function type.");
            if let Type::Function { params, ret } = ft {
                for (arg, param) in args.iter().zip(params.iter()) {
                    match typecheck(ctx, arg) {
                        Some(arg_t) => {
                            if arg_t != *param {
                                panic!("TypeMismatch: Argument {:?} does not match parameter {:?}.", arg_t, param);
                            }
                        },
                        _ => return None,
                    }
                }

                return Some(*ret);
            } else {
                panic!("TypeMismatch: {:?} is not a function.", ft);
            }
        },
        Expr::Builtin(Builtin::AddI(_, _)) => Some(Type::Function(&[Type::Integer, Type::Integer], Type::Integer)),
        Expr::Builtin(Builtin::ExtractField(v, f)) => {
            let t = ctx.get_var_type(v)?;
            match t {
                Type::Enum { kinds } => kinds.get(f).cloned(),
                Type::Struct { fields } => fields.get(f).cloned(),
                Type::Ref(type_id) => {
                    if let Some(t) = ctx.resolve_type(&type_id) {
                        match t {
                            Type::Enum { kinds } => kinds.get(f).cloned(),
                            Type::Struct { fields } => fields.get(f).cloned(),
                            _ => panic!("TypeMismatch: {:?} is not a struct or enum.", t),
                        }
                    } else {
                        panic!("TypeMismatch: {:?} cannot be resolved.", t)
                    }
                },
                _ => panic!("TypeMismatch: {:?} is not a struct or enum.", t),
            }
        },
    }
}

#[cfg(test)]
mod tests {

    use crate::abt::Value;

    use super::*;
    #[test]
    fn test_unit() {
        let ctx = TypecheckContext::new();
        let e = Expr::Value(Value::Unit, None);

        let t = typecheck(&ctx, &e);
        assert!(t.is_some());
        assert_eq!(t.unwrap(), Type::Unit);
    }

    #[test]
    fn test_integer() {
        let ctx = TypecheckContext::new();
        let e = Expr::Value(Value::Integer(42), None);

        let t = typecheck(&ctx, &e);
        assert!(t.is_some());
        assert_eq!(t.unwrap(), Type::Integer);
    }

    #[test]
    fn test_double() {
        let ctx = TypecheckContext::new();
        let e = Expr::Value(Value::Double(3.14), None);

        let t = typecheck(&ctx, &e);
        assert!(t.is_some());
        assert_eq!(t.unwrap(), Type::Double);
    }

    #[test]
    fn test_string() {
        let ctx = TypecheckContext::new();
        let e = Expr::Value(Value::String("hello".to_string()), None);

        let t = typecheck(&ctx, &e);
        assert!(t.is_some());
        assert_eq!(t.unwrap(), Type::String);
    }

    #[test]
    fn test_function() {
        let mut ctx = TypecheckContext::new();
        let e = Expr::Value(Value::Function { params: vec![], body: Box::new(Expr::Value(Value::Integer(42), None)) }, None);
    
        // Function type with no parameters and returns an Integer
        let t0 = TypeId::new();
        let func_type = Type::Function(&[], Type::Integer);
        ctx.add_type(t0, func_type.clone());

        let t = typecheck(&ctx, &e);
        assert!(t.is_some());
        assert_eq!(t.unwrap(), func_type);
    }

    #[test]
    fn test_enum() {
        let mut ctx = TypecheckContext::new();
        // Define an Enum type with a field "Variant" of Integer
        let f0 = FieldId::new();
        let t0 = TypeId::new();
        let enum_type = Type::Enum { kinds: vec![(f0, Type::Integer)].into_iter().collect() };
        ctx.add_type(t0, enum_type.clone());

        // Create an expression representing an instance of the enum
        let e = Expr::Value(Value::Enum { field: f0, value: Box::new(Value::Integer(42)) }, None);

        let t = typecheck(&ctx, &e);
        assert!(t.is_some());
        assert_eq!(t.unwrap(), enum_type);
    }

    #[test]
    fn test_struct() {
        let mut ctx = TypecheckContext::new();
        let f0 = FieldId::new();
        let f1 = FieldId::new();
        let t0 = TypeId::new();
        let struct_type = Type::Struct { fields: vec![(f0, Type::Integer), (f1, Type::Integer)].into_iter().collect() };
        ctx.add_type(t0, struct_type.clone());

        // Create an expression representing an instance of the struct
        let e = Expr::Value(Value::Struct {
            fields: HashMap::from([
                (f0, Box::new(Value::Integer(42))),
                (f1, Box::new(Value::Integer(99)))
            ])
        }, None);

        let t = typecheck(&ctx, &e);
        assert!(t.is_some());
        assert_eq!(t.unwrap(), struct_type);
    }

    #[test]
    fn test_var() {
        let mut ctx = TypecheckContext::new();

        // Define a variable with type Integer
        let var_id = VarId::new();
        let var_type = Type::Integer;
        ctx.add_var(var_id, var_type.clone());

        // Create an expression representing the variable
        let e = Expr::Var(var_id, None);

        let t = typecheck(&ctx, &e);
        assert!(t.is_some());
        assert_eq!(t.unwrap(), var_type);
    }

    #[test]
    fn test_let() {
        let mut ctx = TypecheckContext::new();
        // Define a variable with type Integer
        let v0 = VarId::new();
        let var_type = Type::Integer;
        ctx.add_var(v0, var_type.clone());

        // Create an expression representing the variable in a let binding
        let e = Expr::Let {
            bind: HashMap::from([(v0, Expr::Value(Value::Integer(42), None))]),
            expr: Box::new(Expr::Var(v0, None)),
            debug: None
        };

        let t = typecheck(&ctx, &e);
        assert!(t.is_some());
        assert_eq!(t.unwrap(), var_type);
    }

    #[test]
    fn test_call() {
        let mut ctx = TypecheckContext::new();
        // Define a function with no parameters and returns an Integer
        let v0 = VarId::new();
        let func_type = Type::Function(&[], Type::Integer);
        ctx.add_var(v0, func_type);

        // Create an expression representing the function call
        let e = Expr::Call {
            func: v0,
            args: vec![],
            debug: None
        };

        let t = typecheck(&ctx, &e);
        assert!(t.is_some());
        assert_eq!(t.unwrap(), Type::Integer);
    }

}