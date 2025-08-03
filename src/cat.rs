use anyhow::{Context, Result};
use clap::{Args, ValueHint};
use std::io;

use crate::args::OutputFormat;
use crate::io::{read_data, write_data};

#[derive(Args, Debug)]
pub struct CatArgs {
    /// Input table (file or '-' for stdin)
    #[arg(required = true, value_hint = ValueHint::FilePath)]
    pub table: String,
}

impl CatArgs {
    #[allow(clippy::unused_self)]
    #[allow(clippy::unnecessary_wraps)]
    pub const fn validate(&self) -> io::Result<()> {
        Ok(())
    }

    #[allow(clippy::expect_used)]
    pub fn execute(&self, format: &OutputFormat) -> Result<()> {
        let data = read_data(self.table.as_str(), Some(',')).with_context(|| {
            format!("cat - failed to read csv data from {}", self.table)
        })?;

        write_data(data, format)
            .with_context(|| "cat - failed to write data to stdout".to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_always_succeeds() {
        let args = CatArgs {
            table: "test.csv".to_string(),
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    #[should_panic(expected = "cat - failed to read csv data")]
    #[allow(clippy::unwrap_used)]
    fn test_cat_nonexistent_file_panics() {
        let args = CatArgs {
            table: "nonexistent_file.csv".to_string(),
        };

        args.execute(&crate::args::OutputFormat::Auto).unwrap();
    }

    #[test]
    fn test_cat_orders_csv() {
        let args = CatArgs {
            table: "tests/data/orders/orders.csv".to_string(),
        };

        assert!(args.validate().is_ok());
        assert!(args.execute(&crate::args::OutputFormat::Auto).is_ok());
    }
}
