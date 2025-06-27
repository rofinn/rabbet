use clap::Args;
use std::io;

use crate::args::OutputFormat;
use crate::io::{read_data, write_data};

#[derive(Args, Debug)]
pub struct TailArgs {
    /// Input table (file or '-' for stdin)
    #[arg(required = true)]
    pub table: String,

    /// Number of lines to display from the end
    #[arg(short, long, default_value = "5")]
    pub n: usize,
}

impl TailArgs {
    #[allow(clippy::unused_self)]
    #[allow(clippy::unnecessary_wraps)]
    pub const fn validate(&self) -> io::Result<()> {
        Ok(())
    }

    #[allow(clippy::expect_used)]
    pub fn execute(&self, format: &OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Update read_data to use a circular buffer for better performance
        let data = read_data(self.table.as_str(), Some(',')).expect("Failed to read data");

        let tail_data = data.tail(Some(self.n));

        write_data(tail_data, format)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_always_succeeds() {
        let args = TailArgs {
            table: "test.csv".to_string(),
            n: 5,
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    #[should_panic(expected = "Failed to read data")]
    #[allow(clippy::unwrap_used)]
    fn test_tail_nonexistent_file_panics() {
        let args = TailArgs {
            table: "nonexistent_file.csv".to_string(),
            n: 5,
        };

        args.execute(&crate::args::OutputFormat::Auto).unwrap();
    }

    #[test]
    fn test_tail_basic_csv() {
        let args = TailArgs {
            table: "tests/data/basic/orders.csv".to_string(),
            n: 2,
        };

        assert!(args.validate().is_ok());
        assert!(args.execute(&crate::args::OutputFormat::Auto).is_ok());
    }
}
