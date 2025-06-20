use ariadne::{Cache, Label, Report, Source};
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

        if parse_result.has_errors() {
            let errors = parse_result.into_errors();
            let source = Source::from(&filecontents);

            for error in errors {

                Report::build(ariadne::ReportKind::Error, (file, error.span().into_range()))
                    .with_message(error.reason())
                    .with_label(
                        Label::new((file, error.span().into_range()))
                            .with_message(error.reason())
                    )
                    .finish()
                    .print((file, &source))
                    .unwrap();
            }

            return;
        }

        let ast = parse_result.output().expect("Failed to parse input");
        let ctx = translation::TranslationContext::new();
        let abt = translation::translate(&ctx, ast).expect("Failed to translate AST.");
        let value = naive::eval(&abt);
        println!("{:?}", value);
    }
}
