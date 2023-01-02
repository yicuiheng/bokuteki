mod build;
mod document;
mod katex;
mod lsp;
mod parse;
mod print;
mod util;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct AppArgs {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    Build {
        filepath: String,
        #[clap(short = 'o', long = "output")]
        output_path: Option<String>,
    },
    Lsp,
}

#[tokio::main]
async fn main() {
    init();

    match AppArgs::parse().action {
        Action::Build {
            filepath,
            output_path,
        } => {
            use std::path::PathBuf;
            build::build(
                PathBuf::from(filepath),
                output_path.map_or_else(|| PathBuf::from("./output"), |str| PathBuf::from(str)),
            );
        }
        Action::Lsp => lsp::run().await,
    }
}

fn init() {
    // initialize logger
    simplelog::CombinedLogger::init(vec![simplelog::WriteLogger::new(
        simplelog::LevelFilter::Trace,
        simplelog::Config::default(),
        std::fs::File::create("log.txt").unwrap(),
    )])
    .unwrap();
}
