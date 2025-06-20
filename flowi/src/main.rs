use chumsky::Parser as _;
use clap::Parser as _;
use flow_parser::{ast, translation};

mod naive;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct CliOptions {
    files: Vec<String>,
}

fn main() {
    let cli = CliOptions::parse();

    for file in &cli.files {
        let filecontents = std::fs::read_to_string(file).expect(&format!("Could not read file: {}", file));
        let parse_result = ast::parser().parse(&filecontents);
        let ast = parse_result.output().expect("Failed to parse input");
        let ctx = translation::TranslationContext::new();
        let abt = translation::translate(&ctx, ast).expect("Failed to translate AST.");
        let value = naive::eval(&abt);
        println!("{:?}", value);
    }
}
