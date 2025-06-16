use clap::Parser;

use crate::cat::CatArgs;
use crate::head::HeadArgs;
use crate::join::JoinArgs;
use crate::tail::TailArgs;

#[derive(Parser, Debug)]
#[command(author, version, about = "User-friendly CLI tool for joining tables")]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Cat
    Cat(CatArgs),

    /// Head
    Head(HeadArgs),

    /// Join
    Join(JoinArgs),

    /// Tail
    Tail(TailArgs),
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Commands::Join(join_args) => {
                join_args.validate()?;
                join_args.execute()?;
            }
            Commands::Cat(cat_args) => {
                cat_args.validate()?;
                cat_args.execute()?;
            }
            Commands::Head(head_args) => {
                head_args.validate()?;
                head_args.execute()?;
            }
            Commands::Tail(tail_args) => {
                tail_args.validate()?;
                tail_args.execute()?;
            }
        }
        Ok(())
    }
}
