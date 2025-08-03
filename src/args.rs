use anyhow::Result;
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{Shell, generate};

use crate::aggregate::AggregateArgs;
use crate::cat::CatArgs;
use crate::head::HeadArgs;
use crate::join::JoinArgs;
use crate::query::QueryArgs;
use crate::tail::TailArgs;

#[derive(ValueEnum, Debug, Clone)]
pub enum OutputFormat {
    /// Automatically detect based on terminal (default)
    Auto,
    /// Table format output
    Table,
    /// CSV format output
    Csv,
}

#[derive(Parser, Debug)]
#[command(name = "rabbet", about = "User-friendly CLI tool for joining tables")]
pub struct Args {
    /// Output format
    #[arg(long, value_enum, default_value = "auto", global = true)]
    pub format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Aggregate
    Aggregate(AggregateArgs),

    /// Cat
    Cat(CatArgs),

    /// Head
    Head(HeadArgs),

    /// Join
    Join(JoinArgs),

    /// Query
    Query(QueryArgs),

    /// Tail
    Tail(TailArgs),

    /// Completions
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

impl Args {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Aggregate(aggregate_args) => {
                aggregate_args.validate()?;
                aggregate_args.execute(&self.format)?;
            }
            Commands::Join(join_args) => {
                join_args.validate()?;
                join_args.execute(&self.format)?;
            }
            Commands::Cat(cat_args) => {
                cat_args.validate()?;
                cat_args.execute(&self.format)?;
            }
            Commands::Head(head_args) => {
                head_args.validate()?;
                head_args.execute(&self.format)?;
            }
            Commands::Query(query_args) => {
                query_args.validate()?;
                query_args.execute(&self.format)?;
            }
            Commands::Tail(tail_args) => {
                tail_args.validate()?;
                tail_args.execute(&self.format)?;
            }
            Commands::Completions { shell } => {
                let mut cmd = Self::command();
                generate(*shell, &mut cmd, "rabbet", &mut std::io::stdout());
            }
        }
        Ok(())
    }
}
