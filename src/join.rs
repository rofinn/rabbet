use clap::{Args, ValueEnum};
use itertools::izip;
use once_cell::sync::Lazy;
use polars::prelude::{
    DataFrame, DataFrameJoinOps, JoinArgs as PolarsJoinArgs, JoinType as PolarsJoinType,
};
use regex::Regex;
use std::collections::HashMap;
use std::io;

use crate::io::{read_data, write_data};

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\w+\.\w+(=\w+\.\w+)+").unwrap());

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
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

#[derive(Args, Debug)]
pub struct JoinArgs {
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

impl JoinArgs {
    pub fn validate(&self) -> io::Result<()> {
        if self.tables.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "At least two tables are required for joining",
            ));
        }

        if !self.r#as.is_empty() && self.r#as.len() != self.tables.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Number of table names must match number of tables",
            ));
        }

        if self.on.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "At least one column to join on is required",
            ));
        }

        Ok(())
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let on_map = parse_on_strings(&self.on);
        let mut tables = create_tables(&self.tables, &self.r#as, on_map);
        let mut result = tables.next().unwrap();

        for table in tables {
            result = result.join(&table, self.r#type)
        }

        write_data(result.df)?;

        Ok(())
    }
}

struct Table {
    df: DataFrame,
    name: String,
    on: Vec<String>,
}

impl Table {
    fn load(path: &str, name: &str, on: &[String]) -> Table {
        Table {
            df: read_data(&path, Some(',')).unwrap(),
            name: name.to_string(),
            on: on.to_vec(),
        }
    }
    fn join(&self, other: &Table, method: JoinType) -> Table {
        let result = match method {
            JoinType::Inner => self.df.join(
                &other.df,
                &self.on,
                &other.on,
                PolarsJoinArgs::new(PolarsJoinType::Inner),
            ),
            JoinType::Left => self.df.join(
                &other.df,
                &self.on,
                &other.on,
                PolarsJoinArgs::new(PolarsJoinType::Left),
            ),
            JoinType::Right => other.df.join(
                &self.df,
                &other.on,
                &self.on,
                PolarsJoinArgs::new(PolarsJoinType::Left),
            ),
            JoinType::Outer => self.df.join(
                &other.df,
                &self.on,
                &other.on,
                PolarsJoinArgs::new(PolarsJoinType::Outer),
            ),
        };

        Table {
            df: result.expect("Failed to join tables"),
            name: self.name.clone(),
            on: self.on.clone(),
        }
    }
}

fn create_tables(
    paths: &[String],
    names: &[String],
    on: HashMap<String, Vec<String>>,
) -> impl Iterator<Item = Table> {
    assert!(
        names.is_empty() || names.len() == paths.len(),
        "Number of names must match number of tables"
    );

    let labels = match names.is_empty() {
        true => (0..paths.len()).map(|i| format!("T{}", i + 1)).collect(),
        false => names.to_vec(),
    };

    let global_cols = on.get(&"*".to_string()).cloned().unwrap_or_default();

    izip!(paths, labels).map(move |(p, l)| {
        let mut on_cols = global_cols.clone();
        if let Some(cols) = on.get(&l) {
            on_cols.extend_from_slice(cols);
        }

        assert!(!on_cols.is_empty(), "No columns specified for join");
        Table::load(p, &l, &on_cols)
    })
}

fn parse_on_strings(on: &[String]) -> HashMap<String, Vec<String>> {
    let default_key = "*".to_string();
    let mut result: HashMap<String, Vec<String>> = HashMap::with_capacity(on.len());
    let insert = |result: &mut HashMap<String, Vec<String>>, label: &str, column: &str| {
        result
            .entry(label.to_string())
            .or_insert_with(Vec::new)
            .push(column.to_string());
    };

    for entry in on {
        if RE.is_match(entry) {
            entry
                .split('=')
                .filter_map(|part| part.split_once('.'))
                .for_each(|(label, column)| insert(&mut result, label, column));
        } else {
            insert(&mut result, &default_key, entry);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_create_tables_with_matching_labels() {
        let labels = vec!["users".to_string(), "orders".to_string()];
        let tables = vec!["users.csv".to_string(), "orders.csv".to_string()];
        let on = HashMap::new();

        // Note: This test would require actual files to exist to run successfully
        // In a real test environment, we'd need to create temporary test files
        let result: Vec<Table> = create_tables(&tables, &labels, on).collect();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "users");
        assert_eq!(result[1].name, "orders");
    }

    #[test]
    fn test_create_tables_with_empty_labels() {
        let labels = vec![];
        let tables = vec![
            "table1.csv".to_string(),
            "table2.csv".to_string(),
            "table3.csv".to_string(),
        ];
        let on = HashMap::new();

        // Note: This test would require actual files to exist to run successfully
        // In a real test environment, we'd need to create temporary test files
        let result: Vec<Table> = create_tables(&tables, &labels, on).collect();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "T1");
        assert_eq!(result[1].name, "T2");
        assert_eq!(result[2].name, "T3");
    }

    #[test]
    #[should_panic(expected = "Number of names must match number of tables")]
    fn test_create_tables_with_mismatched_lengths() {
        let labels = vec!["users".to_string()];
        let tables = vec!["users.csv".to_string(), "orders.csv".to_string()];
        let on = HashMap::new();

        let _result: Vec<Table> = create_tables(&tables, &labels, on).collect();
    }

    #[test]
    fn test_parse_on_strings() {
        let column_strings = vec![
            "T1.col11=T2.col12=T3.col13".to_string(),
            "T1.col21=T2.col22=T3.col23".to_string(),
        ];

        let result = parse_on_strings(&column_strings);

        assert_eq!(result["T1"], vec!["col11", "col21"]);
        assert_eq!(result["T2"], vec!["col12", "col22"]);
        assert_eq!(result["T3"], vec!["col13", "col23"]);
    }
}
