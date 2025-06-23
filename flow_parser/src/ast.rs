use chumsky::prelude::*;
use getset::Getters;
use std::collections::HashMap;

use crate::debug::{self, DebugContext, DebugInfo, FileId, Site};

#[derive(Getters)]
#[get = "pub"]
pub struct ParseContext {
    files: HashMap<FileId, String>,
    dbg: DebugContext
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

impl Expr {
    pub fn debug_info(&self) -> &DebugInfo {
        match &self {
            Expr::Value(_, debug_info) => debug_info,
            Expr::Var(_, debug_info) => debug_info,
            Expr::Neg(_, debug_info) => debug_info,
            Expr::Add(_, _, debug_info) => debug_info,
            Expr::Sub(_, _, debug_info) => debug_info,
            Expr::Mul(_, _, debug_info) => debug_info,
            Expr::Div(_, _, debug_info) => debug_info,
            Expr::Function { body, .. } => body.debug_info(),
            Expr::Enum { value, .. } => value.debug_info(),
            Expr::Struct { fields, .. } => {
                fields.values().next().unwrap().debug_info()
            },
            Expr::Let { debug, .. } => debug,
        }
    }
}

impl From<(FileId, SimpleSpan)> for DebugInfo
{
    fn from(value: (FileId, SimpleSpan)) -> Self {
        DebugInfo::new(Site::new(value.0, value.1.into_range()))
    }
}

impl ParseContext {
    pub fn new() -> Self {
        ParseContext {
            files: HashMap::new(),
            dbg: DebugContext::new()
        }
    }

    fn read_file(&mut self, path: &str) -> Result<(FileId, &String), String> {
        let file_contents = std::fs::read_to_string(path).or_else(|e| Err(format!("Could not read file '{}': {}", path, e)))?;
        let file_id = self.dbg.get_file_id(path);
        self.files.insert(file_id, file_contents);
        let file_contents = self.files.get(&file_id).expect("File was just inserted but apparently does not exist.");
        Ok((file_id, file_contents))
    }
}

pub fn parser<'src>(file_id: FileId) -> impl Parser<'src, &'src str, Expr, chumsky::extra::Err<chumsky::error::Rich<'src, char>>> {
    let ident = text::ident()
        .padded();

    let expr = recursive(move |expr| {
        let int = text::int(10)
            .map_with(
                move |s: &str, d|
                Expr::Value(
                    Value::Integer(s.parse().unwrap()),
                    DebugInfo::from((file_id, d.span()))
                )
            )
            .padded();

        let float = text::int(10)
                .then_ignore(just("."))
                .then(text::int(10))
                .map_with(move |(intpart, decimalpart) : (&str, &str), d| {
                    let wholepart = intpart.parse::<f64>().unwrap();
                    let numerator = decimalpart.parse::<f64>().unwrap();
                    let denominator = 10.0f64.powi(decimalpart.len() as i32);
                    Expr::Value(Value::Double(wholepart + (numerator / denominator)), DebugInfo::from((file_id, d.span())))
                })
                .padded();

        let var = text::ident()
                .map_with(move |x: &str, d| Expr::Var(String::from(x), DebugInfo::from((file_id, d.span()))))
                .padded();

        let atom =
            float
            .or(int)
            .or(var)
            .or(expr.clone().delimited_by(just('('), just(')')));

        let op = |c| just(c).padded();

        let unary = op('-')
            .repeated()
            .foldr_with(atom.clone(), move |_op, rhs, d| Expr::Neg(Box::new(rhs), DebugInfo::from((file_id, d.span()))));

        let product = atom.clone()
            .foldl_with(
                op('*').to(Expr::Mul as fn(_, _, _) -> _)
                    .or(op('/').to(Expr::Div as fn(_, _, _) -> _))
                    .then(atom.clone()).repeated(),
                move |lhs, (op, rhs), d| op(Box::new(lhs), Box::new(rhs), DebugInfo::from((file_id, d.span())))
            );

        let sum = product.clone()
            .foldl_with(
                op('+').to(Expr::Add as fn(_, _, _) -> _)
                    .or(op('-').to(Expr::Sub as fn(_, _, _) -> _))
                    .then(product.clone()).repeated(),
                move |lhs, (op, rhs), d| op(Box::new(lhs), Box::new(rhs), DebugInfo::from((file_id, d.span())))
            );

        let r#let = text::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(expr.clone())
            .then_ignore(just(';').or_not())
            .map_with(move |((name, rhs), then): ((&str, Expr), Expr), d| Expr::Let {
                name: String::from(name),
                rhs: Box::new(rhs),
                then: Box::new(then),
                debug: DebugInfo::from((file_id, d.span()))
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

pub fn parse<'src>(ctx: &'src mut ParseContext, file: &str) -> Result<Expr, debug::Error> {
    let (file_id, file_contents) = ctx.read_file(file)?;
    let parse_result: ParseResult<Expr, chumsky::error::Rich<'src, char>> = parser(file_id).parse(&file_contents);

    if parse_result.has_errors() {
        return Err(debug::Error::from_parse_errors(file_id, parse_result.into_errors()));
    } else {
        Ok(parse_result.into_output().expect("Checked for result."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_int() {
        let file = DebugContext::new().get_file_id("");
        let result = parser(file).parse("5");

        match result.unwrap() {
            Expr::Value(Value::Integer(x), _) => assert_eq!(x, 5),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_float() {
        let file = DebugContext::new().get_file_id("");
        let result = parser(file).parse("5.0");

        match result.unwrap() {
            Expr::Value(Value::Double(x), _) => assert_eq!(x, 5.0),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_var() {
        let file = DebugContext::new().get_file_id("");
        let result = parser(file).parse("x");

        match result.unwrap() {
            Expr::Var(name, _) => assert_eq!(name.as_str(), "x"),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_unary() {
        let file = DebugContext::new().get_file_id("");
        let result = parser(file).parse("-5");

        match result.unwrap() {
            Expr::Neg(_, _) => (),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_repeated_unary() {
        let file = DebugContext::new().get_file_id("");
        let result = parser(file).parse("-----5");

        match result.unwrap() {
            Expr::Neg(_, _) => (),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_product() {
        let file = DebugContext::new().get_file_id("");
        let result = parser(file).parse("5 * 5");

        match result.unwrap() {
            Expr::Mul(_, _, _) => (),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_sum() {
        let file = DebugContext::new().get_file_id("");
        let result = parser(file).parse("5+x");

        match result.unwrap() {
            Expr::Add(_, _, _) => (),
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_let() {
        let file = DebugContext::new().get_file_id("");
        let result = parser(file).parse("let x = 5.0; x");

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
        let file = DebugContext::new().get_file_id("");
        let result = parser(file).parse("let x = 5.0; x;");

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
        let file = DebugContext::new().get_file_id("");
        let result = parser(file).parse("
            let x = 5.0;
            let y=3;
            x*y+2
        ");
        result.unwrap();
    }
    
}
