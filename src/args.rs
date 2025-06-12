use clap::{Parser, ValueEnum};
use std::collections::HashMap;
use std::io;

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Outer,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    Default,
    Csv,
    Tsv,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "User-friendly CLI tool for joining tables")]
pub struct Args {
    /// Input tables (files or '-' for stdin)
    #[arg(required = true)]
    pub tables: Vec<String>,

    /// Name your tables
    #[arg(long, value_delimiter = ',')]
    pub r#as: Vec<String>,

    /// Columns to join on (comma separated)
    #[arg(long, value_delimiter = ',')]
    pub on: Vec<String>,

    /// Type of join to perform
    #[arg(long, value_enum, default_value = "inner")]
    pub r#type: JoinType,

    /// Output format
    #[arg(long, value_enum, default_value = "default")]
    pub fmt: OutputFormat,

    /// Delimiter for input files
    #[arg(long, default_value = ",")]
    pub delimiter: char,
}

pub fn validate_args(args: &Args) -> io::Result<()> {
    if args.tables.len() < 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "At least two tables are required for joining",
        ));
    }

    if !args.r#as.is_empty() && args.r#as.len() != args.tables.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Number of table names must match number of tables",
        ));
    }

    if args.on.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "At least one column to join on is required",
        ));
    }

    Ok(())
}

// TODO
// I wonder if we should just have a JoinTable struct which wraps a name, cols and df ?
// We could also encasulate the join match statement to simplify the run function.
pub fn label_tables(tables: &[String], names: &[String]) -> impl Iterator<Item = (String, String)> {
    assert!(
        names.is_empty() || names.len() == tables.len(),
        "Number of names must match number of tables"
    );

    let labels = match names.is_empty() {
        true => (0..tables.len()).map(|i| format!("T{}", i + 1)).collect(),
        false => names.to_vec(),
    };

    return tables
        .iter()
        .enumerate()
        .map(move |(i, table)| (labels[i].clone(), table.clone()));
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_label_tables_with_matching_labels() {
        let labels = vec!["users".to_string(), "orders".to_string()];
        let tables = vec!["users.csv".to_string(), "orders.csv".to_string()];

        let result: HashMap<String, String> = label_tables(&labels, &tables).collect();

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("users"), Some(&"users.csv".to_string()));
        assert_eq!(result.get("orders"), Some(&"orders.csv".to_string()));
    }

    #[test]
    fn test_label_tables_with_empty_labels() {
        let labels = vec![];
        let tables = vec![
            "table1.csv".to_string(),
            "table2.csv".to_string(),
            "table3.csv".to_string(),
        ];

        let result: HashMap<String, String> = label_tables(&labels, &tables).collect();

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("T1"), Some(&"table1.csv".to_string()));
        assert_eq!(result.get("T2"), Some(&"table2.csv".to_string()));
        assert_eq!(result.get("T3"), Some(&"table3.csv".to_string()));
    }

    #[test]
    #[should_panic(expected = "Number of names must match number of tables")]
    fn test_label_tables_with_mismatched_lengths() {
        let labels = vec!["users".to_string()];
        let tables = vec!["users.csv".to_string(), "orders.csv".to_string()];

        let _result: HashMap<String, String> = label_tables(&labels, &tables).collect();
    }
}
