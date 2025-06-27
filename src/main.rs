use clap::Parser;

mod args;
mod cat;
mod head;
mod io;
mod join;
mod query;
mod tail;

use args::Args;
use io::config;

/// The main entry point that parses CLI arguments and runs the join operation
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Configure Polars table formatting based on terminal dimensions and format option
    config(&args.format);
    args.run()
}
