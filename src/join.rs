use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum};
use itertools::izip;
use polars::prelude::{
    DataFrame, DataFrameJoinOps, JoinArgs as PolarsJoinArgs, JoinType as PolarsJoinType,
};
use regex::Regex;
use std::collections::HashMap;

use crate::args::OutputFormat;
use crate::io::{read_data, write_data};

#[allow(clippy::expect_used)]
static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\w+\.\w+(=\w+\.\w+)+").expect("Invalid regex pattern")
});

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Outer,
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

    /// Delimiter for input files
    #[arg(long, default_value = ",")]
    pub delimiter: char,
}

impl JoinArgs {
    pub fn validate(&self) -> Result<()> {
        if self.tables.len() < 2 {
            bail!("At least two tables are required for joining");
        }

        if !self.r#as.is_empty() && self.r#as.len() != self.tables.len() {
            bail!("Number of table names must match number of tables");
        }

        if self.on.is_empty() {
            bail!("At least one column to join on is required");
        }

        Ok(())
    }

    pub fn execute(&self, format: &OutputFormat) -> Result<()> {
        let on_map = parse_on_strings(&self.on);
        let mut tables = create_tables(&self.tables, &self.r#as, &on_map)?;

        if tables.is_empty() {
            bail!("No tables found");
        }

        let mut result = tables.remove(0);

        for table in tables {
            result = result.join(&table, self.r#type)?;
        }

        write_data(result.df, format)?;

        Ok(())
    }
}

struct Table {
    df: DataFrame,
    name: String,
    on: Vec<String>,
}

impl Table {
    fn load(path: &str, name: &str, on: &[String]) -> Result<Self> {
        let df = read_data(path, Some(','))
            .with_context(|| format!("Failed to read table {name} from {path}"))?;

        Ok(Self {
            df,
            name: name.to_string(),
            on: on.to_vec(),
        })
    }

    fn join(&self, other: &Self, method: JoinType) -> Result<Self> {
        let result = match method {
            JoinType::Inner => self.df.join(
                &other.df,
                &self.on,
                &other.on,
                PolarsJoinArgs::new(PolarsJoinType::Inner),
                None,
            ),
            JoinType::Left => self.df.join(
                &other.df,
                &self.on,
                &other.on,
                PolarsJoinArgs::new(PolarsJoinType::Left),
                None,
            ),
            JoinType::Right => other.df.join(
                &self.df,
                &other.on,
                &self.on,
                PolarsJoinArgs::new(PolarsJoinType::Left),
                None,
            ),
            JoinType::Outer => self.df.join(
                &other.df,
                &self.on,
                &other.on,
                PolarsJoinArgs::new(PolarsJoinType::Full),
                None,
            ),
        };

        let df = result.with_context(|| {
            format!("Failed to join '{}' with '{}'", self.name, other.name)
        })?;

        Ok(Self {
            df,
            name: self.name.clone(),
            on: self.on.clone(),
        })
    }
}

fn create_tables(
    paths: &[String],
    names: &[String],
    on: &HashMap<String, Vec<String>>,
) -> Result<Vec<Table>> {
    if !names.is_empty() && names.len() != paths.len() {
        bail!("Number of names must match number of tables");
    }

    let labels = if names.is_empty() {
        (0..paths.len()).map(|i| format!("T{}", i + 1)).collect()
    } else {
        names.to_vec()
    };

    let global_cols = on.get("*").cloned().unwrap_or_default();

    izip!(paths, labels)
        .map(|(p, l)| {
            let mut on_cols = global_cols.clone();
            if let Some(cols) = on.get(&l) {
                on_cols.extend_from_slice(cols);
            }

            if on_cols.is_empty() {
                bail!("No columns specified for join on table '{l}'");
            }

            Table::load(p, &l, &on_cols)
        })
        .collect()
}

fn parse_on_strings(on: &[String]) -> HashMap<String, Vec<String>> {
    let default_key = "*".to_string();
    let mut result: HashMap<String, Vec<String>> = HashMap::with_capacity(on.len());
    let insert = |result: &mut HashMap<String, Vec<String>>, label: &str, column: &str| {
        result
            .entry(label.to_string())
            .or_default()
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_create_tables_with_matching_labels() {
        // Create temporary CSV files
        let mut users_file = NamedTempFile::new().unwrap();
        writeln!(users_file, "id,name,email").unwrap();
        writeln!(users_file, "1,Alice,alice@example.com").unwrap();
        writeln!(users_file, "2,Bob,bob@example.com").unwrap();

        let mut orders_file = NamedTempFile::new().unwrap();
        writeln!(orders_file, "id,user_id,product").unwrap();
        writeln!(orders_file, "101,1,Widget").unwrap();
        writeln!(orders_file, "102,2,Gadget").unwrap();

        let labels = vec!["users".to_string(), "orders".to_string()];
        let tables = vec![
            users_file.path().to_string_lossy().to_string(),
            orders_file.path().to_string_lossy().to_string(),
        ];

        // Provide join columns
        let mut on = HashMap::new();
        on.insert("*".to_string(), vec!["id".to_string()]);

        let result = create_tables(&tables, &labels, &on).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "users");
        assert_eq!(result[1].name, "orders");
        assert_eq!(result[0].on, vec!["id"]);
        assert_eq!(result[1].on, vec!["id"]);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_create_tables_with_empty_labels() {
        // Create temporary CSV files
        let mut table1_file = NamedTempFile::new().unwrap();
        writeln!(table1_file, "id,name").unwrap();
        writeln!(table1_file, "1,Alpha").unwrap();

        let mut table2_file = NamedTempFile::new().unwrap();
        writeln!(table2_file, "id,value").unwrap();
        writeln!(table2_file, "1,100").unwrap();

        let mut table3_file = NamedTempFile::new().unwrap();
        writeln!(table3_file, "id,status").unwrap();
        writeln!(table3_file, "1,Active").unwrap();

        let labels = vec![];
        let tables = vec![
            table1_file.path().to_string_lossy().to_string(),
            table2_file.path().to_string_lossy().to_string(),
            table3_file.path().to_string_lossy().to_string(),
        ];

        // Provide join columns
        let mut on = HashMap::new();
        on.insert("*".to_string(), vec!["id".to_string()]);

        let result = create_tables(&tables, &labels, &on).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "T1");
        assert_eq!(result[1].name, "T2");
        assert_eq!(result[2].name, "T3");
        assert_eq!(result[0].on, vec!["id"]);
        assert_eq!(result[1].on, vec!["id"]);
        assert_eq!(result[2].on, vec!["id"]);
    }

    #[test]
    #[should_panic(expected = "Number of names must match number of tables")]
    #[allow(clippy::unwrap_used)]
    fn test_create_tables_with_mismatched_lengths() {
        // Create temporary CSV files
        let mut users_file = NamedTempFile::new().unwrap();
        writeln!(users_file, "id,name").unwrap();
        writeln!(users_file, "1,Alice").unwrap();

        let mut orders_file = NamedTempFile::new().unwrap();
        writeln!(orders_file, "id,product").unwrap();
        writeln!(orders_file, "1,Widget").unwrap();

        let labels = vec!["users".to_string()];
        let tables = vec![
            users_file.path().to_string_lossy().to_string(),
            orders_file.path().to_string_lossy().to_string(),
        ];

        // Provide join columns
        let mut on = HashMap::new();
        on.insert("*".to_string(), vec!["id".to_string()]);

        let _result = create_tables(&tables, &labels, &on).unwrap();
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

    #[test]
    fn test_join_args_validate_success() {
        let args = JoinArgs {
            tables: vec!["table1.csv".to_string(), "table2.csv".to_string()],
            r#as: vec!["T1".to_string(), "T2".to_string()],
            on: vec!["id".to_string()],
            r#type: JoinType::Inner,
            delimiter: ',',
        };

        assert!(args.validate().is_ok());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_join_args_validate_too_few_tables() {
        let args = JoinArgs {
            tables: vec!["table1.csv".to_string()],
            r#as: vec![],
            on: vec!["id".to_string()],
            r#type: JoinType::Inner,
            delimiter: ',',
        };

        let result = args.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "At least two tables are required for joining"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_join_args_validate_mismatched_table_names() {
        let args = JoinArgs {
            tables: vec!["table1.csv".to_string(), "table2.csv".to_string()],
            r#as: vec!["T1".to_string()], // Only one name for two tables
            on: vec!["id".to_string()],
            r#type: JoinType::Inner,
            delimiter: ',',
        };

        let result = args.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Number of table names must match number of tables"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_join_args_validate_no_join_columns() {
        let args = JoinArgs {
            tables: vec!["table1.csv".to_string(), "table2.csv".to_string()],
            r#as: vec![],
            on: vec![], // No join columns specified
            r#type: JoinType::Inner,
            delimiter: ',',
        };

        let result = args.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "At least one column to join on is required"
        );
    }
}
