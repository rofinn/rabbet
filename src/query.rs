use anyhow::{Context, Result, bail};
use clap::{Args, ValueHint};
use itertools::izip;
use polars::{prelude::IntoLazy, sql::SQLContext};
use std::io::{self, Read};

use crate::args::OutputFormat;
use crate::io::{read_data, write_data};

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Input tables to use in query (files or '-' for stdin)
    #[arg(required = true, value_hint = ValueHint::FilePath)]
    pub tables: Vec<String>,

    /// Name your tables
    #[arg(long, value_delimiter = ',')]
    pub r#as: Vec<String>,

    /// The SQL query to execute (reads from stdin if not provided)
    #[arg(last = true)]
    pub query: Option<String>,
}

impl QueryArgs {
    pub fn validate(&self) -> Result<()> {
        if self.tables.is_empty() {
            bail!("At least one table is required for queries");
        }

        if !self.r#as.is_empty() && self.r#as.len() != self.tables.len() {
            bail!("Number of table names must match number of tables");
        }

        Ok(())
    }

    pub fn execute(&self, format: &OutputFormat) -> Result<()> {
        let mut ctx = SQLContext::new();
        let names = if self.r#as.is_empty() {
            (0..self.tables.len())
                .map(|i| format!("T{}", i + 1))
                .collect()
        } else {
            self.r#as.clone()
        };

        for (name, table) in izip!(names.iter(), self.tables.iter()) {
            ctx.register(
                name,
                read_data(table, None)
                    .with_context(|| format!("query - failed to read table '{table}'"))?
                    .lazy(),
            );
        }

        // Get the query either from the argument or from stdin
        let query = match &self.query {
            Some(q) if q != "-" => q.clone(),
            _ => {
                let mut buffer = String::new();
                io::stdin()
                    .read_to_string(&mut buffer)
                    .context("Failed to read query from stdin")?;
                buffer.trim().to_string()
            }
        };

        if query.is_empty() {
            bail!("Query cannot be empty");
        }
        let result = ctx
            .execute(&query)
            .with_context(|| format!("query - failed to execute query '{query}'"))?
            .collect()
            .with_context(|| "query - failed to collect results".to_string())?;

        write_data(result, format)
            .with_context(|| "query - failed to write data to stdout".to_string())?;

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
            query: Some("SELECT * FROM T1".to_string()),
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validation_no_tables() {
        let args = QueryArgs {
            tables: vec![],
            r#as: vec![],
            query: Some("SELECT * FROM T1".to_string()),
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validation_mismatched_names() {
        let args = QueryArgs {
            tables: vec!["test1.csv".to_string(), "test2.csv".to_string()],
            r#as: vec!["table1".to_string()],
            query: Some("SELECT * FROM table1".to_string()),
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validation_matching_names() {
        let args = QueryArgs {
            tables: vec!["test1.csv".to_string(), "test2.csv".to_string()],
            r#as: vec!["table1".to_string(), "table2".to_string()],
            query: Some("SELECT * FROM table1".to_string()),
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_query_orders_product_filter() {
        let orders_path = "data/orders/orders.csv";

        let args = QueryArgs {
            tables: vec![orders_path.to_string()],
            r#as: vec!["orders".to_string()],
            query: Some(
                "SELECT * FROM orders WHERE product_id = 'PRODUCT-005'".to_string(),
            ),
        };

        assert!(args.validate().is_ok());

        // Test that the query executes without error
        let result = args.execute(&crate::args::OutputFormat::Auto);
        assert!(result.is_ok(), "Query execution should succeed");
    }

    #[test]
    fn test_query_orders_default_table_name() {
        let orders_path = "data/orders/orders.csv";

        let args = QueryArgs {
            tables: vec![orders_path.to_string()],
            r#as: vec![],
            query: Some("SELECT * FROM T1 WHERE product_id = 'PRODUCT-005'".to_string()),
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
