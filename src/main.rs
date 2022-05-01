mod compile;
mod document;
mod katex;
mod lsp;
mod parse;
mod print;
mod source;
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
    match AppArgs::parse().action {
        Action::Compile { filepath } => {
            // initialize logger
            /*
            simplelog::CombinedLogger::init(vec![simplelog::WriteLogger::new(
                simplelog::LevelFilter::Trace,
                simplelog::Config::default(),
                std::fs::File::create("log.txt").unwrap(),
            )])
            .unwrap(); */
            simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
                simplelog::LevelFilter::Trace,
                simplelog::Config::default(),
                simplelog::TerminalMode::Stderr,
                simplelog::ColorChoice::Auto,
            )])
            .unwrap();

            use std::path::PathBuf;
            compile::compile(PathBuf::from(filepath));
        }
        Action::Lsp => {
            // initialize logger
            simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
                simplelog::LevelFilter::Trace,
                simplelog::Config::default(),
                simplelog::TerminalMode::Stderr,
                simplelog::ColorChoice::Auto,
            )])
            .unwrap();

            lsp::run().await
        }
    }
}
