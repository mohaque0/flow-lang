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
    String(String)
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value, DebugInfo),
    Var(String, DebugInfo),

    Neg(Box<Expr>, DebugInfo),
    Add(Box<Expr>, Box<Expr>, DebugInfo),
    Sub(Box<Expr>, Box<Expr>, DebugInfo),
    Mul(Box<Expr>, Box<Expr>, DebugInfo),
    Div(Box<Expr>, Box<Expr>, DebugInfo),

    Function { args: Vec<String>, body: Box<Expr> },
    Enum { name: String, value: Box<Expr> },
    Struct { name: String, fields: HashMap<String, Box<Expr>> },

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
            .or(expr.clone().delimited_by(just('('), just(')')));

        let op = |c| just(c).padded();

        let unary = op('-')
            .repeated()
            .foldr_with(atom.clone(), |_op, rhs, d| Expr::Neg(Box::new(rhs), DebugInfo::from(d.span())));

        let product = atom.clone()
            .foldl_with(
                op('*').to(Expr::Mul as fn(_, _, _) -> _)
                    .or(op('/').to(Expr::Div as fn(_, _, _) -> _))
                    .then(atom.clone()).repeated(),
                |lhs, (op, rhs), d| op(Box::new(lhs), Box::new(rhs), DebugInfo::from(d.span()))
            );

        let sum = product.clone()
            .foldl_with(
                op('+').to(Expr::Add as fn(_, _, _) -> _)
                    .or(op('-').to(Expr::Sub as fn(_, _, _) -> _))
                    .then(product.clone()).repeated(),
                |lhs, (op, rhs), d| op(Box::new(lhs), Box::new(rhs), DebugInfo::from(d.span()))
            );

        let r#let = text::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(expr.clone())
            .then_ignore(just(';').or_not())
            .map_with(|((name, rhs), then): ((&str, Expr), Expr), d| Expr::Let {
                name: String::from(name),
                rhs: Box::new(rhs),
                then: Box::new(then),
                debug: DebugInfo::from(d.span())
            });

        r#let
            .or(sum)
            .or(product)
            .or(unary)
            .or(atom)
            .padded()
    });

    expr
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

    #[test]
    fn parse_float() {
        let result = parser().parse("5.0");

        match result.unwrap() {
            Expr::Value(Value::Double(x), _) => assert_eq!(x, 5.0),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_var() {
        let result = parser().parse("x");

        match result.unwrap() {
            Expr::Var(name, _) => assert_eq!(name.as_str(), "x"),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_unary() {
        let result = parser().parse("-5");

        match result.unwrap() {
            Expr::Neg(_, _) => (),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_repeated_unary() {
        let result = parser().parse("-----5");

        match result.unwrap() {
            Expr::Neg(_, _) => (),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_product() {
        let result = parser().parse("5 * 5");

        match result.unwrap() {
            Expr::Mul(_, _, _) => (),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_sum() {
        let result = parser().parse("5+x");

        match result.unwrap() {
            Expr::Add(_, _, _) => (),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_let() {
        let result = parser().parse("let x = 5.0; x");

        match result.unwrap() {
            Expr::Let { name, rhs, then, debug: _} => {
                assert_eq!(name.as_str(), "x");
                match *rhs {
                    Expr::Value(Value::Double(x), _) => assert_eq!(x, 5.0),
                    _ => assert!(false)
                }
                match *then {
                    Expr::Var(name, _) => assert_eq!(name.as_str(), "x"),
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_let_with_semicolon() {
        let result = parser().parse("let x = 5.0; x;");

        match result.unwrap() {
            Expr::Let { name, rhs, then, debug: _} => {
                assert_eq!(name.as_str(), "x");
                match *rhs {
                    Expr::Value(Value::Double(x), _) => assert_eq!(x, 5.0),
                    _ => assert!(false)
                }
                match *then {
                    Expr::Var(name, _) => assert_eq!(name.as_str(), "x"),
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_test_program0() {
        let result = parser().parse("
            let x = 5.0;
            let y=3;
            x*y+2
        ");
        result.unwrap();
    }
    
}
