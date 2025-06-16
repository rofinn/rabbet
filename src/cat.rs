use clap::Args;
use std::io;

use crate::io::{read_data, write_data};

#[derive(Args, Debug)]
pub struct CatArgs {
    /// Input table (file or '-' for stdin)
    #[arg(required = true)]
    pub table: String,
}

impl CatArgs {
    #[allow(clippy::unused_self)]
    #[allow(clippy::unnecessary_wraps)]
    pub const fn validate(&self) -> io::Result<()> {
        Ok(())
    }

    #[allow(clippy::expect_used)]
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        write_data(
            read_data(self.table.as_str(), Some(',')).expect("Failed to read data"),
        )?;

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
    #[should_panic(expected = "Failed to read data")]
    #[allow(clippy::unwrap_used)]
    fn test_cat_nonexistent_file_panics() {
        let args = CatArgs {
            table: "nonexistent_file.csv".to_string(),
        };

        args.execute().unwrap();
    }

    #[test]
    fn test_cat_basic_csv() {
        let args = CatArgs {
            table: "tests/data/basic/orders.csv".to_string(),
        };

        assert!(args.validate().is_ok());
        assert!(args.execute().is_ok());
    }
}
