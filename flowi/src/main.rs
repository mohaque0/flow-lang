use std::path::{Path, PathBuf};

use ariadne::{Cache as _, FileCache, FnCache, Label, Report, Source};
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

    let filename = &cli.file;

    let mut ctx = ParseContext::new();

    let result = parse(&mut ctx, &filename);

    if let Err(errors) = result {
        match errors {
            flow_parser::debug::Error::Simple(e) => {
                println!("{}", e);
            },
            flow_parser::debug::Error::Site(site_errors) => {
                for error in site_errors {

                    let source = Source::from(ctx.files().get(&error.site().file()).expect("Unknown file id."));

                    Report::build(ariadne::ReportKind::Error, (filename, error.site().span().clone()))
                        .with_message(error.reason())
                        .with_label(
                            Label::new((filename, error.site().span().clone()))
                                .with_message(error.reason())
                        )
                        .finish()
                        .print((filename, &source))
                        .unwrap();
                }
            },
        }

        return;
    }

    // We already checked errors. This shouldn't happen.
    let ast = result.expect("Unexpected error parsing input.");

    let pctx = ctx;
    let ctx = translation::TranslationContext::new();
    let abt = match translation::translate(&ctx, &ast) {
        Ok(value) => value,
        Err(error) => {
            match error {
                translation::TranslationError::UndefinedOperation { op, a, b } => {

                    let mut cache = FileCache::default();
                    let mut filename_a = None;
                    let mut filename_b = None;
                    if let Some(s) = a.clone() {
                        if let Some(filename) = pctx.dbg().get_file_path(&s.file()) {
                            cache.fetch(Path::new(&filename));
                            filename_a = Some(filename);
                        }
                    }
                    if let Some(s) = b.clone() {
                        if let Some(filename) = pctx.dbg().get_file_path(&s.file()) {
                            cache.fetch(Path::new(&filename));
                            filename_b = Some(filename);
                        }
                    }

                    if filename_a.is_some() && filename_b.is_some() {
                        let filename_a = filename_a.unwrap();
                        let filename_b = filename_b.unwrap();

                        let cache = FnCache::new(|path: &String| {
                            std::fs::read_to_string(path)
                        });

                        let a = a.unwrap();
                        let b = b.unwrap();

                        Report::build(ariadne::ReportKind::Error, (filename_a.clone(), a.span().clone()))
                            .with_message(format!("undefined operation: {op}"))
                            .with_labels(
                                [
                                    Label::new((filename_a, a.span().clone()))
                                        .with_message("between this"),
                                    Label::new((filename_b, b.span().clone()))
                                        .with_message("and this"),
                                ]
                            )
                            .finish()
                            .print(cache)
                            .unwrap();
                    } else {
                        todo!()
                    }
                },
                translation::TranslationError::UnknownVariable(_, site) => {

                },
            }
            return;
        }
    };

    let value = naive::eval(&abt);
    println!("{:?}", value);

}
