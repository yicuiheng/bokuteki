mod document;
mod lsp;
mod parse;
mod print;
mod util;

use clap::{Parser, Subcommand};
use std::fs;

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
            let src = fs::read_to_string(filepath.clone()).expect("failed to read file..");
            let src: Vec<Vec<char>> = src.lines().map(|line| line.chars().collect()).collect();
            let src_block_range = document::src_block_range(&src);

            let result = parse::parse_block_elements(&src, src_block_range);
            print::print(&src, result.value);
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
