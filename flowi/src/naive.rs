use std::collections::HashMap;

use flow_parser::abt::{Expr, Value, VarId};

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
        Expr::Var(var_id) => Expr::Value(ctx.var(var_id).expect("Compiler must guarantee vars are defined."), None),
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
            let func = ctx.var(func).expect("Compiler must guarantee vars are defined.");
            if let Value::Function { params, body } = func {
                if args.len() != params.len() {
                    panic!("Incorrect number of args for function.");
                }

                let mut bindings = HashMap::new();
                params.iter().zip(args).for_each(|(var_id, var_def)| {
                    bindings.insert(*var_id, var_def.clone());
                });

                Expr::Let { bind: bindings, expr: body.clone(), debug: None }

            } else {
                panic!("Type mismatch. Function was not a function.");
            }
        },
    }
}

pub fn eval(e: Expr) -> Value {
    let mut ctx = EvalContextImpl::new();
    let mut e = e;
    loop {
        if let Expr::Value(v, _d) = e {
            return v
        } else {
            e = reduce(&mut ctx, &e);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_int() {
        let expr = Expr::Let { 
            bind: HashMap::from_iter([(VarId(0), Expr::Value(Value::Unit, None))]),
            expr: Box::new(Expr::Value(Value::Unit, None)),
            debug: None
        };

        println!("{:?}", eval(expr));
    }
}