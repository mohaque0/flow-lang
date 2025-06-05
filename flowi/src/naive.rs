use std::collections::HashMap;

use flow_parser::abt::{Builtin, Expr, Value, VarId};

pub trait EvalContext {
    fn var(&self, var_id: &VarId) -> Option<Value>;
    fn set_var(&mut self, var_id: &VarId, value: Value);
    fn scoped(&self) -> EvalContextImpl;
}

#[derive(Clone)]
pub struct EvalContextImpl {
    vars: HashMap<VarId, Value>
}

impl EvalContextImpl {
    fn new() -> Self {
        Self {
            vars: HashMap::new()
        }
    }
}

impl EvalContext for EvalContextImpl  {
    fn var(&self, var_id: &VarId) -> Option<Value> {
        if let Some(v) = self.vars.get(&var_id) {
            return Some(v.clone());
        } else {
            return None;
        }
    }

    fn set_var(&mut self, var_id: &VarId, value: Value) {
        self.vars.insert(*var_id, value);
    }
    
    fn scoped(&self) -> EvalContextImpl {
        self.clone()
    }
}

fn reduce(ctx: &dyn EvalContext, e: &Expr) -> Expr {
    match e {
        Expr::Value(value, debug_info) => Expr::Value(value.clone(), debug_info.clone()),
        Expr::Var(var_id) => Expr::Value(ctx.var(var_id).expect(&format!("Compiler must guarantee vars are defined {:?}", var_id)), None),
        Expr::Let { bind, expr, .. } => {
            let mut bindings = HashMap::new();
            let mut reduced_bindings = 0;
            let mut unresolved_bindings = 0;
            
            let mut scoped_ctx = ctx.scoped();
            for (var_id, var_def) in bind {
                if let Expr::Value(v, _) = var_def {
                    scoped_ctx.set_var(var_id, v.clone());
                }
            }

            for (var_id, var_def) in bind {
                if var_def.unbound_variables().iter().all(|it| scoped_ctx.var(it).is_some()) {
                    let reduced_expr = reduce(&scoped_ctx, var_def);
                    reduced_bindings += 1;
                    unresolved_bindings += if let Expr::Value(_, _) = reduced_expr { 0 } else { 1 };
                    bindings.insert(*var_id, reduced_expr);
                } else {
                    unresolved_bindings += 1;
                }
            }

            if unresolved_bindings > 0 {
                if reduced_bindings == 0 {
                    panic!("Let expression cannot be reduced.")
                }
                Expr::Let { bind: bindings, expr: expr.clone(), debug: None }
            } else {
                let mut scoped_ctx = ctx.scoped();

                bindings.iter().for_each(|(var_id, value)| {
                    let value = if let Expr::Value(v, _) = value { v } else { panic!("Bindings evaluated but found unevaluated binding.") };
                    scoped_ctx.set_var(var_id, value.clone())
                });
                return reduce(&scoped_ctx, expr);
            }
        },
        Expr::Call { func, args, .. } => {
            let func = ctx.var(func).expect(&format!("Compiler must guarantee vars are defined {:?}", func));
            let func = Expr::Value(func, None);
            let func = func.with_fresh_vars(&|var| ctx.var(var).is_some());

            let args: Vec<Expr> = args.iter()
                .map(|it| it.with_fresh_vars(&|var| ctx.var(var).is_some()))
                .collect();

            if let Expr::Value(Value::Function { params, body }, _) = func {
                if args.len() != params.len() {
                    panic!("Incorrect number of args for function.");
                }

                let mut bindings = HashMap::new();
                params.iter().zip(args).for_each(|((var_id, _), var_def)| {
                    for var in var_def.unbound_variables() {
                        bindings.insert(var, Expr::Value(ctx.var(&var).expect(&format!("Unknown var {:?}", var)), None));
                    }
                    bindings.insert(*var_id, var_def.clone());
                });

                for var in body.unbound_variables() {
                    if !bindings.contains_key(&var) {
                        bindings.insert(var, Expr::Value(ctx.var(&var).expect(&format!("Unknown var {:?}", var)), None));
                    }
                }

                Expr::Let { bind: bindings, expr: Box::new(*body), debug: None }

            } else {
                panic!("Type mismatch. Function was not a function.");
            }
        },
        Expr::Builtin(Builtin::AddI(v1, v2)) => {
            let v1 = ctx.var(v1).expect("Undefined var.");
            let v2 = ctx.var(v2).expect("Undefined var.");

            let v1 = if let Value::Integer(i) = v1 { i } else { panic!("Type mismatch.") };
            let v2 = if let Value::Integer(i) = v2 { i } else { panic!("Type mismatch.") };

            Expr::Value(Value::Integer(v1 + v2), None)
        },
    }
}

pub fn eval(e: &Expr) -> Value {
    let mut ctx = EvalContextImpl::new();
    let mut e = e.clone();
    println!("{:?}", e);
    loop {
        if let Expr::Value(v, _d) = e {
            return v.clone()
        } else {
            e = reduce(&mut ctx, &e);
            println!("{:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {

    use flow_parser::abt::Type;

    use super::*;

    #[test]
    fn test_let_binding() {
        let v0 = VarId::new();

        let expr = Expr::Let { 
            bind: HashMap::from_iter([(v0, Expr::Value(Value::Unit, None))]),
            expr: Box::new(Expr::Var(v0)),
            debug: None
        };

        let value = eval(&expr);

        println!("{:?}", value);

        assert_eq!(value, Value::Unit);
    }

    #[test]
    fn test_builtin_addi() {
        let v0 = VarId::new();
        let v1 = VarId::new();
        let v2 = VarId::new();
        let v3 = VarId::new();

        let expr = Expr::Let { 
            bind: HashMap::from_iter([
                (v0, Expr::Value(Value::Function {
                    params: Vec::from([(v2, Type::Integer), (v3, Type::Integer)]),
                    body: Box::new(Expr::Builtin(Builtin::AddI(v2, v3)))
                }, None)),
                (v1, Expr::Value(Value::Integer(1), None)),
                (v2, Expr::Value(Value::Integer(2), None))
            ]),
            expr: Box::new(Expr::Call {
                func: v0,
                args: Vec::from([Expr::Var(v1), Expr::Var(v2)]),
                debug: None
            }),
            debug: None
        };

        let value = eval(&expr);

        println!("{:?}", value);

        assert_eq!(value, Value::Integer(3));
    }

        #[test]
    fn test_nested_function() {
        let v0 = VarId::new();
        let v1 = VarId::new();
        let v2 = VarId::new();
        let v3 = VarId::new();

        let expr = Expr::Let { 
            bind: HashMap::from_iter([
                (v0, Expr::Value(Value::Function {
                    params: Vec::from([(v2, Type::Integer), (v3, Type::Integer)]),
                    body: Box::new(Expr::Builtin(Builtin::AddI(v2, v3)))
                }, None)),
                (v1, Expr::Value(Value::Integer(1), None)),
                (v2, Expr::Value(Value::Integer(2), None))
            ]),
            expr: Box::new(Expr::Call {
                func: v0,
                args: Vec::from([
                    Expr::Call {
                        func: v0,
                        args: Vec::from([
                            Expr::Var(v1),
                            Expr::Var(v2)
                        ]),
                        debug: None
                    },
                    Expr::Var(v2)
                ]),
                debug: None
            }),
            debug: None
        };

        let value = eval(&expr);

        println!("{:?}", value);

        assert_eq!(value, Value::Integer(5));
    }
}