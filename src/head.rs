// Same behaviour as `head` in Unix, but pretty printed with polars.
use anyhow::{Context, Result};
use clap::{Args, ValueHint};
use std::io;

use crate::args::OutputFormat;
use crate::io::{read_data, write_data};

#[derive(Args, Debug)]
pub struct HeadArgs {
    /// Input table (file or '-' for stdin)
    #[arg(required = true, value_hint = ValueHint::FilePath)]
    pub table: String,

    /// Number of lines to display from the beginning
    #[arg(short, long, default_value = "5")]
    pub n: usize,
}

impl HeadArgs {
    #[allow(clippy::unused_self)]
    #[allow(clippy::unnecessary_wraps)]
    pub const fn validate(&self) -> io::Result<()> {
        Ok(())
    }

    #[allow(clippy::expect_used)]
    pub fn execute(&self, format: &OutputFormat) -> Result<()> {
        let data = read_data(self.table.as_str(), Some(',')).with_context(|| {
            format!("head - failed to read csv data from {}", self.table)
        })?;

        let head_data = data.head(Some(self.n));

        write_data(head_data, format)
            .with_context(|| "head - failed to write data to stdout".to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_always_succeeds() {
        let args = HeadArgs {
            table: "test.csv".to_string(),
            n: 5,
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    #[should_panic(expected = "head - failed to read csv data")]
    #[allow(clippy::unwrap_used)]
    fn test_head_nonexistent_file_panics() {
        let args = HeadArgs {
            table: "nonexistent_file.csv".to_string(),
            n: 5,
        };

        args.execute(&crate::args::OutputFormat::Auto).unwrap();
    }

    #[test]
    fn test_head_orders_csv() {
        let args = HeadArgs {
            table: "tests/data/orders/orders.csv".to_string(),
            n: 2,
        };

        assert!(args.validate().is_ok());
        assert!(args.execute(&crate::args::OutputFormat::Auto).is_ok());
    }
}
