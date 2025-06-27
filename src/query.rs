use clap::Args;
use itertools::izip;
use polars::{prelude::IntoLazy, sql::SQLContext};
use std::io;

use crate::args::OutputFormat;
use crate::io::{read_data, write_data};

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Input tables to use in query (files or '-' for stdin)
    #[arg(required = true)]
    pub tables: Vec<String>,

    /// Name your tables
    #[arg(long, value_delimiter = ',')]
    pub r#as: Vec<String>,

    /// The SQL query to execute
    #[arg(required = true, last = true)]
    pub query: String,
}

impl QueryArgs {
    pub fn validate(&self) -> io::Result<()> {
        if self.tables.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "At least one table is required for queries",
            ));
        }

        if !self.r#as.is_empty() && self.r#as.len() != self.tables.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Number of table names must match number of tables",
            ));
        }

        Ok(())
    }

    pub fn execute(&self, format: &OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
        let mut ctx = SQLContext::new();
        let names = if self.r#as.is_empty() {
            (0..self.tables.len())
                .map(|i| format!("T{}", i + 1))
                .collect()
        } else {
            self.r#as.clone()
        };

        for (name, table) in izip!(names.iter(), self.tables.iter()) {
            ctx.register(name, read_data(table, None)?.lazy());
        }

        let result = ctx.execute(&self.query)?.collect()?;

        write_data(result, format)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let args = QueryArgs {
            tables: vec!["test.csv".to_string()],
            r#as: vec![],
            query: "SELECT * FROM T1".to_string(),
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validation_no_tables() {
        let args = QueryArgs {
            tables: vec![],
            r#as: vec![],
            query: "SELECT * FROM T1".to_string(),
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validation_mismatched_names() {
        let args = QueryArgs {
            tables: vec!["test1.csv".to_string(), "test2.csv".to_string()],
            r#as: vec!["table1".to_string()],
            query: "SELECT * FROM table1".to_string(),
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validation_matching_names() {
        let args = QueryArgs {
            tables: vec!["test1.csv".to_string(), "test2.csv".to_string()],
            r#as: vec!["table1".to_string(), "table2".to_string()],
            query: "SELECT * FROM table1".to_string(),
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_query_orders_product_filter() {
        let orders_path = "tests/data/basic/orders.csv";

        let args = QueryArgs {
            tables: vec![orders_path.to_string()],
            r#as: vec!["orders".to_string()],
            query: "SELECT * FROM orders WHERE product_id = 'PRODUCT-005'".to_string(),
        };

        assert!(args.validate().is_ok());

        // Test that the query executes without error
        let result = args.execute(&crate::args::OutputFormat::Auto);
        assert!(result.is_ok(), "Query execution should succeed");
    }

    #[test]
    fn test_query_orders_default_table_name() {
        let orders_path = "tests/data/basic/orders.csv";

        let args = QueryArgs {
            tables: vec![orders_path.to_string()],
            r#as: vec![],
            query: "SELECT * FROM T1 WHERE product_id = 'PRODUCT-005'".to_string(),
        };

        assert!(args.validate().is_ok());

        // Test that the query executes without error using default table name
        let result = args.execute(&crate::args::OutputFormat::Auto);
        assert!(
            result.is_ok(),
            "Query execution with default table name should succeed"
        );
    }
}
