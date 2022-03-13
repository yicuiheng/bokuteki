mod compile;
mod document;
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
    Compile { filepath: String },
    Lsp,
}

#[tokio::main]
async fn main() {
    init();

    match AppArgs::parse().action {
        Action::Compile { filepath } => {
            use std::path::PathBuf;
            compile::compile(PathBuf::from(filepath));
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
