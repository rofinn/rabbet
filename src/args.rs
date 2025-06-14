use clap::Parser;

use crate::join::JoinArgs;

#[derive(Parser, Debug)]
#[command(author, version, about = "User-friendly CLI tool for joining tables")]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Join
    Join(JoinArgs),
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Commands::Join(join_args) => {
                join_args.validate()?;
                join_args.execute()?;
            }
        }
        Ok(())
    }
}
