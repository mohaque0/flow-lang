use std::collections::HashMap;

use crate::{abt::{self, Type, VarId}, ast, debug::Site, typecheck::{typecheck, TypecheckContext}};

#[derive(Clone)]
pub struct TranslationContext {
    typechecking_context: TypecheckContext,
    scoped_vars: HashMap<String, VarId>,
    operators: HashMap<OperatorType, VarId>,
}

impl TranslationContext {
    pub fn new() -> Self {
        return TranslationContext {
            typechecking_context: TypecheckContext::new(),
            scoped_vars: HashMap::new(),
            operators: HashMap::new(),
        };
    }

    fn get_typechecking_context(&self) -> &TypecheckContext {
        return &self.typechecking_context;
    }

    fn fresh_var(&mut self) -> VarId {
        return VarId::new();
    }

    fn set_var(&mut self, name: &String, id: VarId) -> Option<VarId> {
        self.scoped_vars.insert(name.clone(), id)
    }

    fn get_var(&self, name: &String) -> Option<VarId> {
        self.scoped_vars.get(name).cloned()
    }

    fn get_op(&self, op: &OperatorType) -> Option<VarId> {
        self.operators.get(op).cloned()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OperatorType {
    Neg(Type),
    Add(Type, Type),
    Sub(Type, Type),
    Mul(Type, Type),
    Div(Type, Type),
}

#[derive(Clone, Debug)]
pub enum TranslationError {
    UndefinedOperation { op: &'static str, a: Option<Site>, b: Option<Site>, ta: Option<Type>, tb: Option<Type> },
    UnknownVariable(String, Site),
}

impl TranslationError {
    fn UndefinedOperation(op: &'static str, e0: &abt::Expr, e1: &abt::Expr, t0: Option<Type>, t1: Option<Type>) -> Self {
        TranslationError::UndefinedOperation {
            op,
            a: e0.debug_info().map(|it| it.site().clone()).or(None),
            b: e1.debug_info().map(|it| it.site().clone()).or(None),
            ta: t0,
            tb: t1,
        }
    }
}


pub fn translate(ctx: &TranslationContext, ast: &ast::Expr) -> Result<abt::Expr, TranslationError> {
    match ast {
        ast::Expr::Value(value, debug_info) => {
            let value = match value {
                ast::Value::Integer(v) => abt::Value::Integer(*v),
                ast::Value::Double(v) => abt::Value::Double(*v),
                ast::Value::String(v) => abt::Value::String(v.clone()),
            };

            Ok(abt::Expr::Value(value, Some(debug_info.clone())))
        },
        ast::Expr::Var(v, debug_info) => {
            ctx.get_var(v)
                .map(|id| abt::Expr::Var(id, Some(debug_info.clone())))
                .ok_or_else(|| TranslationError::UnknownVariable(v.clone(), debug_info.site().clone()))
        },
        ast::Expr::Neg(expr, debug_info) => todo!(),
        ast::Expr::Add(e0, e1, debug_info) => {
            let e0 = translate(ctx, e0)?;
            let e1 = translate(ctx, e1)?;

            let t0 = typecheck(ctx.get_typechecking_context(), &e0);
            let t1 = typecheck(ctx.get_typechecking_context(), &e1);

            if t0 == None || t1 == None {
                return Err(TranslationError::UndefinedOperation("+", &e0, &e1, t0, t1));
            }

            let (t0, t1) = (t0.expect("Checked."), t1.expect("Checked."));
            
            if let Some(op) = ctx.get_op(&OperatorType::Add(t0.clone(), t1.clone())) {
                Ok(abt::Expr::Call { func: op, args: vec![e0, e1], debug: None })
            } else {
                Err(TranslationError::UndefinedOperation("+", &e0, &e1, Some(t0), Some(t1)))
            }
        },
        ast::Expr::Sub(e0, e1, debug_info) => todo!(),
        ast::Expr::Mul(e0, e1, debug_info) => todo!(),
        ast::Expr::Div(e0, e1, debug_info) => todo!(),
        ast::Expr::Function { args, body } => todo!(),
        ast::Expr::Enum { name, value } => todo!(),
        ast::Expr::Struct { name, fields } => todo!(),
        ast::Expr::Let { name, rhs, then, debug } => {

            let rhs = translate(&ctx, rhs)?;

            let var_id = VarId::new();
            let var_def = rhs;

            let mut subctx: TranslationContext = ctx.clone();
            subctx.set_var(name, var_id); // TODO: Must add to typechecking context as well.

            let then = translate(&subctx, then)?;

            Ok(abt::Expr::Let {
                bind: HashMap::from([(var_id, var_def)]),
                expr: Box::new(then),
                debug: None
            })
        },
    }
}


// pub fn translate(ast: &ast::Expr) -> abt::Expr {
    
// }