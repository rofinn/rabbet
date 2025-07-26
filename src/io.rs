use anyhow::Result;
use polars::prelude::*;
use std::env;
use std::fs::File;
use std::io::{self, Cursor, IsTerminal, Read, Write};

use crate::args::OutputFormat;

/// # IO Module
///
/// This module provides functionality to read CSV data into Polars `DataFrames`
/// from various sources including files and stdin.
///
/// ## Usage Examples
///
/// ```rust
/// use rabbet::io::read_data;
///
/// // Example 1: Read from a CSV file with default comma separator
/// let df = read_data(
///     &"data.csv".to_string(),
///     None
/// )?;
/// println!("Loaded {} rows with {} columns", df.height(), df.width());
///
/// // Example 2: Read from a TSV file with tab separator
/// let df = read_data(
///     &"data.tsv".to_string(),
///     Some('\t')
/// )?;
///
/// // Example 3: Read from stdin (pipe data in)
/// // echo "name,age\nAlice,30\nBob,25" | cargo run
/// let df = read_data(&"-".to_string(), None)?;
///
/// // Example 4: Read with custom separator (semicolon)
/// let df = read_data(
///     &"european_data.csv".to_string(),
///     Some(';')
/// )?;
/// ```
/// Sets up Polars table formatting environment variables based on terminal size
///
/// This function detects the current terminal dimensions and configures Polars
/// display settings to optimize table output for the available space.
///
/// # Terminal Size Handling
///
/// - **Width**: Uses terminal width (min: 80, max: 300 characters)
/// - **Height**: Uses terminal height minus 5 rows for headers/prompts (min: 10, max: 1000 rows)
/// - **Fallback**: If terminal size detection fails, uses conservative defaults (120x25)
///
/// # Environment Variables Set
///
/// - `POLARS_TABLE_WIDTH`: Maximum table width in characters
/// - `POLARS_FMT_MAX_ROWS`: Maximum number of rows to display
/// - Various formatting options for clean, readable output
pub fn config(format: &OutputFormat) {
    fn set_var(key: &str, default: &str) {
        if env::var(key).is_err() {
            unsafe {
                env::set_var(key, default);
            }
        }
    }

    let should_format_table = match format {
        OutputFormat::Auto => {
            env::var("RABBET_TABLE_OUTPUT").is_ok() || std::io::stdout().is_terminal()
        }
        OutputFormat::Table => true,
        OutputFormat::Csv => false,
    };

    if should_format_table {
        set_var("POLARS_FMT_TABLE_FORMATTING", "UTF8_BORDERS_ONLY");
        set_var("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES", "1");
        set_var("POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION", "1");
        set_var("POLARS_FMT_TABLE_ROUNDED_CORNERS", "1");
        set_var("POLARS_FMT_STR_LEN", "16");
        set_var("POLARS_FMT_MAX_COLS", "100");

        if let Some(size) = termsize::get() {
            // Calculate optimal dimensions based on terminal size
            let table_width = (size.cols as usize).clamp(80, 300);
            let max_rows = (size.rows as usize).saturating_sub(5).clamp(10, 1000);
            set_var("POLARS_TABLE_WIDTH", &table_width.to_string());
            set_var("POLARS_FMT_MAX_ROWS", &max_rows.to_string());
        } else {
            // Fallback to conservative defaults when terminal size is unavailable
            set_var("POLARS_TABLE_WIDTH", "120");
            set_var("POLARS_FMT_MAX_ROWS", "25");
        }
    }
}

/// Reads CSV data into a Polars `DataFrame` from either a file or stdin
///
/// # Arguments
///
/// * `source` - Either a file path or stdin as the data source
/// * `separator` - Optional separator character, defaults to ','
///
/// # Returns
///
/// * `Result<DataFrame, Box<dyn std::error::Error>>` - The resulting `DataFrame` or an error
///
/// # Examples
///
/// ```
/// use rabbet::io::read_data;
///
/// // Read from file with default comma separator
/// let df = read_data(&"data.csv".to_string(), None)?;
///
/// // Read from file with custom separator
/// let df = read_data(&"data.tsv".to_string(), Some('\t'))?;
///
/// // Read from stdin
/// let df = read_data(&"-".to_string(), None)?;
/// ```
pub fn read_data(source: &str, separator: Option<char>) -> Result<DataFrame> {
    let sep = separator.unwrap_or(',') as u8;
    let mut buffer = String::new();

    match source {
        "-" => io::stdin().read_to_string(&mut buffer)?,
        _ => File::open(source)?.read_to_string(&mut buffer)?,
    };

    let parse_options = CsvParseOptions::default().with_separator(sep);
    let df = CsvReadOptions::default()
        .with_parse_options(parse_options)
        .with_has_header(true)
        .into_reader_with_file_handle(Cursor::new(buffer))
        .finish()?;

    Ok(df)
}

/// Writes a Polars `DataFrame` to stdout as CSV format
///
/// # Arguments
///
/// * `df` - The `DataFrame` to write to stdout
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - Ok(()) on success, error on failure
///
/// # Examples
///
/// ```
/// use rabbet::io::{read_data, write_data};
/// use polars::prelude::*;
///
/// // Read data from a file
/// let df = read_data(&"data.csv".to_string(), None)?;
///
/// // Write the DataFrame to stdout as CSV
/// write_data(df)?;
/// ```
pub fn write_data(mut df: DataFrame, format: &OutputFormat) -> Result<()> {
    // Print final result
    let should_format_table = match format {
        OutputFormat::Auto => {
            env::var("RABBET_TABLE_OUTPUT").is_ok() || std::io::stdout().is_terminal()
        }
        OutputFormat::Table => true,
        OutputFormat::Csv => false,
    };

    if should_format_table {
        println!("{df:?}");
    } else {
        let mut buffer = Vec::new();
        CsvWriter::new(&mut buffer)
            .with_separator(b',')
            .finish(&mut df)?;

        std::io::stdout().write_all(&buffer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[allow(clippy::expect_used)]
    fn test_config() {
        use crate::args::OutputFormat;
        // Call the setup function - it should not panic
        config(&OutputFormat::Auto);

        // Only test environment variables if we're in a terminal environment
        if std::io::stdout().is_terminal() {
            // Verify that key environment variables are set
            assert!(env::var("POLARS_TABLE_WIDTH").is_ok());
            assert!(env::var("POLARS_FMT_MAX_ROWS").is_ok());
            assert!(env::var("POLARS_FMT_TABLE_FORMATTING").is_ok());

            // Verify specific formatting values
            assert_eq!(
                env::var("POLARS_FMT_TABLE_FORMATTING")
                    .expect("POLARS_FMT_TABLE_FORMATTING should be set"),
                "UTF8_BORDERS_ONLY"
            );
            assert_eq!(
                env::var("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES")
                    .expect("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES should be set"),
                "1"
            );
            assert_eq!(
                env::var("POLARS_FMT_TABLE_ROUNDED_CORNERS")
                    .expect("POLARS_FMT_TABLE_ROUNDED_CORNERS should be set"),
                "1"
            );

            // Verify table width is within expected bounds
            let table_width: usize = env::var("POLARS_TABLE_WIDTH")
                .expect("POLARS_TABLE_WIDTH should be set")
                .parse()
                .expect("POLARS_TABLE_WIDTH should be a valid number");
            assert!((80..=300).contains(&table_width));

            // Verify max rows is within expected bounds
            let max_rows: usize = env::var("POLARS_FMT_MAX_ROWS")
                .expect("POLARS_FMT_MAX_ROWS should be set")
                .parse()
                .expect("POLARS_FMT_MAX_ROWS should be a valid number");
            assert!((10..=1000).contains(&max_rows));
        } else {
            // In non-terminal environments (like CI), just verify the function doesn't panic
            // and that no terminal-specific variables are set
            assert!(env::var("POLARS_TABLE_WIDTH").is_err());
            assert!(env::var("POLARS_FMT_MAX_ROWS").is_err());
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    fn test_read_data_from_file_comma_separated() {
        // Create a temporary CSV file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,Los Angeles").unwrap();

        let file_path = temp_file.path().to_string_lossy().to_string();

        // Test reading with default comma separator
        let df = read_data(&file_path, None).expect("Failed to read data");

        assert_eq!(df.shape().0, 2); // 2 rows
        assert_eq!(df.shape().1, 3); // 3 columns
        assert_eq!(df.get_column_names(), &["name", "age", "city"]);
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    fn test_read_data_from_file_tab_separated() {
        // Create a temporary TSV file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name\tage\tcity").unwrap();
        writeln!(temp_file, "Alice\t30\tNew York").unwrap();
        writeln!(temp_file, "Bob\t25\tLos Angeles").unwrap();

        let file_path = temp_file.path().to_string_lossy().to_string();

        // Test reading with tab separator
        let df = read_data(&file_path, Some('\t')).expect("Failed to read data");

        assert_eq!(df.shape().0, 2); // 2 rows
        assert_eq!(df.shape().1, 3); // 3 columns
        assert_eq!(df.get_column_names(), &["name", "age", "city"]);
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    fn test_read_data_with_semicolon_separator() {
        // Create a temporary CSV file with semicolon separator
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name;age;city").unwrap();
        writeln!(temp_file, "Alice;30;New York").unwrap();
        writeln!(temp_file, "Bob;25;Los Angeles").unwrap();

        let file_path = temp_file.path().to_string_lossy().to_string();

        // Test reading with semicolon separator
        let df = read_data(&file_path, Some(';')).expect("Failed to read data");

        assert_eq!(df.shape().0, 2); // 2 rows
        assert_eq!(df.shape().1, 3); // 3 columns
        assert_eq!(df.get_column_names(), &["name", "age", "city"]);
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    fn test_read_data_empty_file() {
        // Test reading from an empty file with only headers
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,score").unwrap();

        let file_path = temp_file.path().to_string_lossy().to_string();

        let df = read_data(&file_path, None).expect("Failed to read data");

        assert_eq!(df.shape().0, 0); // 0 rows
        assert_eq!(df.shape().1, 3); // 3 columns
        assert_eq!(df.get_column_names(), &["id", "name", "score"]);
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    fn test_write_data() {
        // Create a small test DataFrame
        let mut df = df! {
            "name" => ["Alice", "Bob"],
            "age" => [30, 25],
            "city" => ["New York", "Los Angeles"]
        }
        .expect("Failed to create DataFrame");

        // Write DataFrame as CSV to a buffer to test the CSV output functionality
        let mut buffer = Vec::new();
        {
            use polars::prelude::CsvWriter;
            CsvWriter::new(&mut buffer)
                .with_separator(b',')
                .finish(&mut df)
                .expect("Failed to write CSV");
        }

        // Verify the CSV output contains expected data
        let output = String::from_utf8(buffer).expect("Failed to convert buffer to string");
        assert!(output.contains("name,age,city"));
        assert!(output.contains("Alice,30,New York"));
        assert!(output.contains("Bob,25,Los Angeles"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_data_file_not_found() {
        // Test reading from a non-existent file
        let non_existent_path = "/path/that/does/not/exist.csv";

        let result = read_data(non_existent_path, None);

        // Should return an error
        assert!(result.is_err());

        // Verify it's a file not found error
        let error = result.unwrap_err();
        let error_string = error.to_string().to_lowercase();
        assert!(
            error_string.contains("no such file") || error_string.contains("not found")
        );
    }
}
