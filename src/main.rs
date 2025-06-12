use clap::{Parser, parser};
use polars::prelude::{DataFrameJoinOps, JoinArgs, JoinType as PolarsJoinType};

mod args;
mod io;
mod join;

use args::{Args, JoinType, label_tables, validate_args};
use io::{config, read_data, write_data};
use join::parse_columns;

/// The main entry point that parses CLI arguments and runs the join operation
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure Polars table formatting based on terminal dimensions
    config();

    // Parse command line arguments
    let args = Args::parse();

    // Run the join operation with parsed arguments
    run(args)?;
    Ok(())
}

/// Core function to run table joins based on provided arguments.
///
/// This function is public to allow for direct testing without invoking the CLI.
pub fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Validate basic arguments
    validate_args(&args)?;

    let on_map = parse_columns(&args.r#as, &args.on);
    let mut tables = label_tables(&args.r#as, &args.tables);

    // Read the first table as the base
    let (llabel, ltable) = tables.next().unwrap();
    let mut result = read_data(&ltable, Some(','))?;

    // Iteratively join with remaining tables
    for (rlabel, rtable) in tables {
        let df = read_data(&rtable, Some(','))?;

        // Join with accumulated result
        // TODO: Use a static mapping to avoid duplicated code.
        result = match args.r#type {
            JoinType::Inner => result.join(
                &df,
                &on_map[&llabel],
                &on_map[&rlabel],
                JoinArgs::new(PolarsJoinType::Inner),
            )?,
            JoinType::Left => result.join(
                &df,
                &on_map[&llabel],
                &on_map[&rlabel],
                JoinArgs::new(PolarsJoinType::Left),
            )?,
            JoinType::Right => df.join(
                &result,
                &on_map[&rlabel],
                &on_map[&llabel],
                JoinArgs::new(PolarsJoinType::Left),
            )?,
            JoinType::Outer => result.join(
                &df,
                &on_map[&llabel],
                &on_map[&rlabel],
                JoinArgs::new(PolarsJoinType::Outer),
            )?,
        };
    }

    write_data(result)?;

    Ok(())
}
