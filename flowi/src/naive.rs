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
        Expr::Let { bind, expr, debug } => {
            let mut bindings = HashMap::new();
            let mut unresolved_bindings = 0;
            
            for (var_id, var_def) in bind {
                let mut scoped_ctx = ctx.scoped();
                let reduced_expr = reduce(&mut scoped_ctx, var_def);
                unresolved_bindings += if let Expr::Value(_, _) = reduced_expr { 0 } else { 1 };
                bindings.insert(*var_id, reduced_expr);
            }

            if unresolved_bindings > 0 {
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
        Expr::Call { func, args, debug } => {
            let func = ctx.var(func).expect(&format!("Compiler must guarantee vars are defined {:?}", func));
            if let Value::Function { params, body } = func {
                if args.len() != params.len() {
                    panic!("Incorrect number of args for function.");
                }

                let mut bindings = HashMap::new();
                params.iter().zip(args).for_each(|((var_id, _type), var_def)| {
                    bindings.insert(*var_id, var_def.clone());
                });

                Expr::Let { bind: bindings, expr: body.clone(), debug: None }

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
    loop {
        if let Expr::Value(v, _d) = e {
            return v.clone()
        } else {
            e = reduce(&mut ctx, &e);
        }
    }
}

#[cfg(test)]
mod tests {

    use flow_parser::abt::Type;

    use super::*;

    #[test]
    fn test_let_binding() {
        let expr = Expr::Let { 
            bind: HashMap::from_iter([(VarId(0), Expr::Value(Value::Unit, None))]),
            expr: Box::new(Expr::Var(VarId(0))),
            debug: None
        };

        let value = eval(&expr);

        //assert_eq!(value, Value::Unit);

        println!("{:?}", value);
    }

    #[test]
    fn test_builtin_addi() {
        let expr = Expr::Let { 
            bind: HashMap::from_iter([
                (VarId(0), Expr::Value(Value::Function {
                    params: Vec::from([(VarId(2), Type::Integer), (VarId(3), Type::Integer)]),
                    body: Box::new(Expr::Builtin(Builtin::AddI(VarId(2), VarId(3))))
                }, None)),
                (VarId(1), Expr::Value(Value::Integer(1), None)),
                (VarId(2), Expr::Value(Value::Integer(2), None))
            ]),
            expr: Box::new(Expr::Call { func: VarId(0), args: Vec::from([Expr::Var(VarId(1)), Expr::Var(VarId(2))]), debug: None }),
            debug: None
        };

        println!("{:?}", eval(&expr));
    }
}