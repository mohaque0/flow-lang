use chumsky::prelude::*;
use derive_more::Constructor;
use std::{collections::HashMap, ops::Range};

#[derive(Debug, Clone, Constructor)]
pub struct DebugInfo {
    span: Range<usize>
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Double(f64),
    Function(Box<Expr>),
    Enum { name: String, value: Box<Value> },
    Struct { name: String, fields: HashMap<String, Box<Value>> }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value, DebugInfo),
    Var(String, DebugInfo),
    Let {
        name: String,
        rhs: Box<Expr>,
        then: Box<Expr>,
        debug: DebugInfo
    },
}

impl From<SimpleSpan> for DebugInfo
{
    fn from(value: SimpleSpan) -> Self {
        DebugInfo { span: value.into_range() }
    }
}

pub fn parser<'src>() -> impl Parser<'src, &'src str, Expr> {
    let ident = text::ident()
        .padded();

    let expr = recursive(|expr| {
        let int = text::int(10)
            .map_with(
                |s: &str, d|
                Expr::Value(
                    Value::Integer(s.parse().unwrap()),
                    DebugInfo::from(d.span()))
                )
            .padded();

        let float = text::int(10)
                .then_ignore(just("."))
                .then(text::int(10))
                .map_with(|(intpart, decimalpart) : (&str, &str), d| {
                    let wholepart = intpart.parse::<f64>().unwrap();
                    let numerator = decimalpart.parse::<f64>().unwrap();
                    let denominator = 10.0f64.powi(decimalpart.len() as i32);
                    Expr::Value(Value::Double(wholepart + (numerator / denominator)), DebugInfo::from(d.span()))
                })
                .padded();

        let var = text::ident()
                .map_with(|x: &str, d| Expr::Var(String::from(x), DebugInfo::from(d.span())))
                .padded();

        let atom =
            float
            .or(int)
            .or(var)
            .or(expr.delimited_by(just('('), just(')')));

        atom
    });

    let decl = recursive(|decl| {
        let r#let = text::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl.clone())
            .map_with(|((name, rhs), then): ((&str, Expr), Expr), d| Expr::Let {
                name: String::from(name),
                rhs: Box::new(rhs),
                then: Box::new(then),
                debug: DebugInfo::from(d.span())
            });
    
        r#let
            .or(expr)
            .padded()
    });    


    decl
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_int() {
        let result = parser().parse("5");

        match result.unwrap() {
            Expr::Value(Value::Integer(x), _) => assert_eq!(x, 5),
            _ => assert!(false)
        }
    }

    fn parse_float() {
        let result = parser().parse("5.0");

        match result.unwrap() {
            Expr::Value(Value::Double(x), _) => assert_eq!(x, 5.0),
            _ => assert!(false)
        }
    }
}
