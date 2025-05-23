use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use chumsky::Parser;
use flow_parser::{parser, Expr, Value};

fn eval<'a>(
    expr: &'a Expr,
    vars: &mut Vec<(&'a String, Value)>,
    funcs: &mut Vec<(&'a String, &'a [String], &'a Expr)>,
) -> Result<Value, String> {
    match expr {
        Expr::Value(x) => Ok(x.clone()),
        Expr::Neg(a) => Ok(
            match eval(a, vars, funcs)? {
                Value::Integer(i) => Value::Integer(-i),
                Value::Double(f) => Value::Double(-f),
                Value::Function(_) => return Err(String::from("Cannot negate a function.")),
            }
        ),
        Expr::Add(a, b) => {
            let a = eval(a, vars, funcs)?;
            let b = eval(b, vars, funcs)?;
            return numeric_op(|a,b| a+b, |a, b| a+b, a, b);
        },
        Expr::Sub(a, b) => {
            let a = eval(a, vars, funcs)?;
            let b = eval(b, vars, funcs)?;
            return numeric_op(|a,b| a-b, |a, b| a-b, a, b);
        },
        Expr::Mul(a, b) => {
            let a = eval(a, vars, funcs)?;
            let b = eval(b, vars, funcs)?;
            return numeric_op(|a,b| a*b, |a, b| a*b, a, b);
        },
        Expr::Div(a, b) => {
            let a = eval(a, vars, funcs)?;
            let b = eval(b, vars, funcs)?;
            return numeric_op(|a,b| a/b, |a, b| a/b, a, b);
        },
        Expr::Var(name) => if let Some((_, val)) = vars.iter().rev().find(|(var, _)| *var == name) {
            Ok(val.clone())
        } else {
            Err(format!("Cannot find variable `{}` in scope", name))
        },
        Expr::Let { name, rhs, then } => {
            let rhs = eval(rhs, vars, funcs)?;
            vars.push((name, rhs));
            let output = eval(then, vars, funcs);
            vars.pop();
            output
        },
        Expr::Call(name, args) => if let Some((_, arg_names, body)) = funcs
            .iter()
            .rev()
            .find(|(var, _, _)| *var == name)
            .copied()
        {
            if arg_names.len() == args.len() {
                let mut args = args
                    .iter()
                    .map(|arg| eval(arg, vars, funcs))
                    .zip(arg_names.iter())
                    .map(|(val, name)| Ok((name, val?)))
                    .collect::<Result<_, String>>()?;
                vars.append(&mut args);
                let output = eval(body, vars, funcs);
                vars.truncate(vars.len() - args.len());
                output
            } else {
                Err(format!(
                    "Wrong number of arguments for function `{}`: expected {}, found {}",
                    name,
                    arg_names.len(),
                    args.len(),
                ))
            }
        } else {
            Err(format!("Cannot find function `{}` in scope", name))
        },
        Expr::Fn { name, args, body, then } => {
            funcs.push((name, args, body));
            let output = eval(then, vars, funcs);
            funcs.pop();
            output
        },
    }
}

fn numeric_op<Intop, Flop>(intop: Intop, flop: Flop, a: Value, b: Value) -> Result<Value, String>
    where
      Intop: FnOnce(i64, i64) -> i64,
      Flop: FnOnce(f64, f64) -> f64
{
    if let Value::Integer(a) = a {
        if let Value::Integer(b) = b {
            return Ok(Value::Integer(intop(a, b)));
        }
    }
    if let Value::Double(a) = a {
        if let Value::Double(b) = b {
            return Ok(Value::Double(flop(a, b)));
        }
    }
    return Err(String::from("Types must matach."));
}


fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let src = std::fs::read_to_string(filename.clone()).unwrap();
    let expr = parser().parse(src.clone());

    if let Err(err) = expr {
        let filename = filename.as_str();
        let mut colors = ColorGenerator::new();
        let a = colors.next();

        for e in err {

            Report::build(ReportKind::Error, filename, 12)
                .with_code(3)
                .with_message(format!("{:?}", e.reason()))
                .with_label(
                    Label::new((filename, e.span()))
                        .with_message(format!("{:?}", e.reason()))
                        .with_color(a),
                )
                .finish()
                .print((filename, Source::from(src.clone())))
                .unwrap();
    

            println!("Error {:?}: {:?} Expected: {:?} but found: {:?}", e.span(), e.reason(), e.expected().collect::<Vec<_>>(), e.found());
        }
        return;
    }

    let result = eval(&expr.unwrap(), &mut Vec::new(), &mut Vec::new());

    println!("{:?}", result);
}