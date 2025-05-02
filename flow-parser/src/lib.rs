use chumsky::prelude::*;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Double(f64),
    Function(Box<Expr>)
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value),
    Var(String),

    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    Call(String, Vec<Expr>),
    Let {
        name: String,
        rhs: Box<Expr>,
        then: Box<Expr>,
    },
    Fn {
        name: String,
        args: Vec<String>,
        body: Box<Expr>,
        then: Box<Expr>,
    },
}


pub fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let ident = text::ident()
        .padded();

    let expr = recursive(|expr| {
        let int = text::int(10)
            .map(|s: String| Expr::Value(Value::Integer(s.parse().unwrap())))
            .padded();

        let float = text::int::<char, Simple<char>>(10)
                .then_ignore(just("."))
                .then(text::int(10))
                .map_with_span(|(intpart, decimalpart), _| {
                    let wholepart = intpart.parse::<f64>().unwrap();
                    let numerator = decimalpart.parse::<f64>().unwrap();
                    let denominator = 10.0f64.powi(decimalpart.len() as i32);
                    Expr::Value(Value::Double(wholepart + (numerator / denominator)))
                })
                .padded();

        let call = ident
            .then(expr.clone()
                .separated_by(just(','))
                .allow_trailing() // Foo is Rust-like, so allow trailing commas to appear in arg lists
                .delimited_by(just('('), just(')')))
            .map(|(f, args)| Expr::Call(f, args));

        let atom =
            float
            .or(int)
            .or(expr.delimited_by(just('('), just(')')))
            .or(call)
            .or(ident.map(Expr::Var));

        let op = |c| just(c).padded();

        let unary = op('-')
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

        let product = unary.clone()
            .then(op('*').to(Expr::Mul as fn(_, _) -> _)
                .or(op('/').to(Expr::Div as fn(_, _) -> _))
                .then(unary)
                .repeated())
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let sum = product.clone()
            .then(op('+').to(Expr::Add as fn(_, _) -> _)
                .or(op('-').to(Expr::Sub as fn(_, _) -> _))
                .then(product)
                .repeated())
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        sum
    });

    let decl = recursive(|decl| {
        let r#let = text::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl.clone())
            .map(|((name, rhs), then)| Expr::Let {
                name,
                rhs: Box::new(rhs),
                then: Box::new(then),
            });
    
        let r#fn = text::keyword("fn")
            .ignore_then(ident)
            .then_ignore(just("(").padded())
            .then(ident.separated_by(just(','))
            .allow_trailing())
            .then_ignore(just(")").padded())
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl)
            .map(|(((name, args), body), then)| Expr::Fn {
                name,
                args,
                body: Box::new(body),
                then: Box::new(then),
            });
    
        r#let
            .or(r#fn)
            .or(expr)
            .padded()
    });    

    decl
        .then_ignore(end())
}