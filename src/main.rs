use clap::Parser;

mod args;
mod cat;
mod head;
mod io;
mod join;
mod tail;

use args::Args;
use io::config;

/// The main entry point that parses CLI arguments and runs the join operation
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure Polars table formatting based on terminal dimensions
    config();

    // Parse command line arguments
    let args = Args::parse();
    args.run()
}
