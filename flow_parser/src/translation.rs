use std::{collections::HashMap, sync::{atomic::AtomicUsize, Arc}};

use crate::{abt::{self, VarId}, ast};

struct TranslationContext {
    scoped_vars: HashMap<String, VarId>,
    next_var_id: Arc<AtomicUsize>
}

impl TranslationContext {
    fn fresh_var(&mut self) -> VarId {
        return VarId(self.next_var_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    }

    fn set_var(&mut self, name: &String, id: VarId) -> Option<VarId> {
        self.scoped_vars.insert(name.clone(), id)
    }
}


fn translate(ctx: &mut TranslationContext, ast: &ast::Expr) -> abt::Expr {
    match ast {
        ast::Expr::Value(value, debug_info) => {
            let value = match value {
                ast::Value::Integer(v) => abt::Value::Integer(*v),
                ast::Value::Double(v) => abt::Value::Double(*v),
                ast::Value::String(v) => abt::Value::String(v.clone()),
            };

            abt::Expr::Value(value, None)
        },
        ast::Expr::Var(v, debug_info) => todo!(),
        ast::Expr::Neg(expr, debug_info) => todo!(),
        ast::Expr::Add(expr, expr1, debug_info) => todo!(),
        ast::Expr::Sub(expr, expr1, debug_info) => todo!(),
        ast::Expr::Mul(expr, expr1, debug_info) => todo!(),
        ast::Expr::Div(expr, expr1, debug_info) => todo!(),
        ast::Expr::Function { args, body } => todo!(),
        ast::Expr::Enum { name, value } => todo!(),
        ast::Expr::Struct { name, fields } => todo!(),
        ast::Expr::Let { name, rhs, then, debug } => todo!(),
    }
}


// pub fn translate(ast: &ast::Expr) -> abt::Expr {
    
// }