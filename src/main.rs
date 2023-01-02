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
    Build { filepath: String },
    Lsp,
}

#[tokio::main]
async fn main() {
    init();

    match AppArgs::parse().action {
        Action::Build { filepath } => {
            use std::path::PathBuf;
            build::build(PathBuf::from(filepath));
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
