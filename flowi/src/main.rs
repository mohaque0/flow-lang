use ariadne::{Label, Report, Source};
use clap::Parser as _;
use flow_parser::{ast::{parse, ParseContext}, translation};

mod naive;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct CliOptions {
    file: String,
}

fn main() {
    let cli = CliOptions::parse();

    let file = &cli.file;

    let mut ctx = ParseContext::new();

    let result = parse(&mut ctx, &file);

    if let Err(errors) = result {
        match errors {
            flow_parser::debug::Error::Simple(e) => {
                println!("{}", e);
            },
            flow_parser::debug::Error::Site(site_errors) => {
                for error in site_errors {

                    let source = Source::from(ctx.files().get(&error.site().file()).expect("Unknown file id."));

                    Report::build(ariadne::ReportKind::Error, (file, error.site().span().clone()))
                        .with_message(error.reason())
                        .with_label(
                            Label::new((file, error.site().span().clone()))
                                .with_message(error.reason())
                        )
                        .finish()
                        .print((file, &source))
                        .unwrap();
                }
            },
        }

        return;
    }

    // We already checked errors. This shouldn't happen.
    let ast = result.expect("Unexpected error parsing input.");

    let ctx = translation::TranslationContext::new();
    let abt = match translation::translate(&ctx, &ast) {
        Ok(value) => value,
        Err(error) => {
            match error {
                translation::TranslationError::UndefinedOperation => todo!(),
                translation::TranslationError::UnknownVariable(_) => todo!(),
            }
            return;
        }
    };

    let value = naive::eval(&abt);
    println!("{:?}", value);

}
