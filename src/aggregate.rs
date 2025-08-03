use anyhow::{Context, Result, bail, ensure};
use clap::{Args, ValueHint};
use polars::prelude::*;

use crate::args::OutputFormat;
use crate::io::{read_data, write_data};

#[derive(Args, Debug)]
pub struct AggregateArgs {
    /// Input table (file or '-' for stdin)
    #[arg(required = true, value_hint = ValueHint::FilePath)]
    pub table: String,

    /// Columns to group by (comma separated)
    ///
    /// Examples: --by "category" or --by "category,region"
    #[arg(long, value_delimiter = ',')]
    pub by: Vec<String>,

    /// Aggregation operations as column=operation pairs (comma separated)
    ///
    /// Operations: sum, mean, median, min, max, range, count, variance, stddev, first, last, describe
    ///
    /// Examples:
    /// - Single aggregation: --with "amount=sum"
    /// - Multiple aggregations: --with "amount=sum,price=mean,quantity=max"
    /// - Row-based operations: --with "_=count" (counts rows)
    /// - Multiple ops on same column: --with "price=min,price=max,price=mean"
    #[arg(long, value_delimiter = ',')]
    pub with: Vec<String>,

    /// Delimiter for input files
    #[arg(long, default_value = ",")]
    pub delimiter: char,
}

impl AggregateArgs {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.with.is_empty(),
            "At least one aggregation operation must be specified with --with"
        );

        // Validate aggregation operations
        let valid_ops = [
            "sum", "mean", "median", "min", "max", "range", "count", "len", "nrow",
            "variance", "stddev", "first", "last", "describe",
        ];

        for spec in &self.with {
            let parts: Vec<&str> = spec.split('=').collect();
            ensure!(
                parts.len() == 2,
                "Invalid aggregation specification '{}'. Expected format: column=operation",
                spec
            );

            let column = parts[0];
            let operation = parts[1];
            ensure!(
                valid_ops.contains(&operation),
                "Invalid operation '{}'. Valid operations: {}",
                operation,
                valid_ops.join(", ")
            );

            if column == "_" {
                ensure!(
                    matches!(operation, "count" | "len" | "nrow"),
                    "Invalid operation '{}'. '_' can only be used with row-based operations: count, len, nrow",
                    operation,
                );
            }
        }

        Ok(())
    }

    pub fn execute(&self, format: &OutputFormat) -> Result<()> {
        // Read input data
        let df = read_data(&self.table, Some(self.delimiter))
            .with_context(|| format!("Failed to read data from {}", self.table))?;

        // Parse aggregation specifications
        let aggs = parse_aggs(&self.with)?;

        // Perform aggregation
        let result: LazyFrame = if self.by.is_empty() {
            df.lazy().select(aggs)
        } else {
            let cols: Vec<_> = self.by.iter().map(std::string::String::as_str).collect();
            df.lazy().group_by_stable(cols).agg(aggs)
        };

        // Write output
        write_data(
            result.collect().with_context(|| {
                format!("Failed to perform aggregation on {}", self.table)
            })?,
            format,
        )
        .with_context(|| "Failed to write aggregated data to stdout")?;

        Ok(())
    }
}

fn parse_aggs(with_strs: &[String]) -> Result<Vec<Expr>> {
    let mut aggs: Vec<Expr> = Vec::new();

    for spec in with_strs {
        let parts: Vec<&str> = spec.split('=').collect();
        let column = parts[0];
        let operation = parts[1];
        let alias = format!("{column}_{operation}");
        let expr = match (column, operation) {
            ("_", "count") => len().alias("count"),
            ("_", "len") => len().alias("len"),
            ("_", "nrow") => len().alias("nrow"),
            (_, "sum") => col(column).sum().alias(&alias),
            (_, "mean") => col(column).mean().alias(&alias),
            (_, "median") => col(column).median().alias(&alias),
            (_, "min") => col(column).min().alias(&alias),
            (_, "max") => col(column).max().alias(&alias),
            (_, "first") => col(column).first().alias(&alias),
            (_, "last") => col(column).last().alias(&alias),
            (_, "range") => (col(column).max() - col(column).min()).alias(&alias),
            (_, "count") | ("len" | "nrow", _) => col(column).count().alias(&alias),
            (_, "variance") => col(column).var(1).alias(&alias), // Use sample variance (ddof=1)
            (_, "stddev") => col(column).std(1).alias(&alias), // Use sample std dev (ddof=1)
            (_, "describe") => {
                // For describe, we'll create a concatenated string of statistics
                // This is a simplified version - in a real implementation you might want
                // to return multiple columns or a structured result
                concat_str(
                    [
                        lit("count: "),
                        col(column).clone().count().cast(DataType::String),
                        lit(", mean: "),
                        col(column).mean().cast(DataType::String),
                        lit(", std: "),
                        col(column).std(1).cast(DataType::String),
                        lit(", min: "),
                        col(column).min().cast(DataType::String),
                        lit(", max: "),
                        col(column).max().cast(DataType::String),
                    ],
                    "",
                    false,
                )
                .alias(&alias)
            }
            (_, _) => bail!("Unsupported operation: {}", operation),
        };

        aggs.push(expr);
    }
    Ok(aggs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_no_operations() {
        let args = AggregateArgs {
            table: "test.csv".to_string(),
            by: vec![],
            with: vec![],
            delimiter: ',',
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_operation() {
        let args = AggregateArgs {
            table: "test.csv".to_string(),
            by: vec![],
            with: vec!["col=invalid".to_string()],
            delimiter: ',',
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_valid_operations() {
        let args = AggregateArgs {
            table: "test.csv".to_string(),
            by: vec!["group".to_string()],
            with: vec!["value=sum".to_string(), "count=count".to_string()],
            delimiter: ',',
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_underscore_with_count() {
        let args = AggregateArgs {
            table: "test.csv".to_string(),
            by: vec!["group".to_string()],
            with: vec!["_=count".to_string()],
            delimiter: ',',
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_underscore_with_invalid_op() {
        let args = AggregateArgs {
            table: "test.csv".to_string(),
            by: vec![],
            with: vec!["_=mean".to_string()],
            delimiter: ',',
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_first_last_operations() {
        let args = AggregateArgs {
            table: "test.csv".to_string(),
            by: vec!["group".to_string()],
            with: vec!["value=first".to_string(), "other=last".to_string()],
            delimiter: ',',
        };
        assert!(args.validate().is_ok());
    }
}
