use clap::Parser;

mod args;
mod io;
mod join;

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
