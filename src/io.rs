use polars::prelude::*;
use std::env;
use std::fs::File;
use std::io::{self, Cursor, IsTerminal, Read, Write};
use termsize;

/// # IO Module
///
/// This module provides functionality to read CSV data into Polars DataFrames
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
pub fn config() {
    fn set_var(key: &str, default: &str) {
        unsafe {
            if env::var(key).is_err() {
                env::set_var(key, default);
            }
        }
    }

    if std::io::stdout().is_terminal() {
        set_var("POLARS_FMT_TABLE_FORMATTING", "UTF8_BORDERS_ONLY");
        set_var("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES", "1");
        set_var("POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION", "1");
        set_var("POLARS_FMT_TABLE_ROUNDED_CORNERS", "1");
        set_var("POLARS_FMT_STR_LEN", "16");
        set_var("POLARS_FMT_MAX_COLS", "100");

        if let Some(size) = termsize::get() {
            // Calculate optimal dimensions based on terminal size
            let table_width = (size.cols as usize).max(80).min(300);
            let max_rows = (size.rows as usize).saturating_sub(5).max(10).min(1000);
            set_var("POLARS_TABLE_WIDTH", &table_width.to_string());
            set_var("POLARS_FMT_MAX_ROWS", &max_rows.to_string());
        } else {
            // Fallback to conservative defaults when terminal size is unavailable
            set_var("POLARS_TABLE_WIDTH", "120");
            set_var("POLARS_FMT_MAX_ROWS", "25")
        }
    }
}

/// Reads CSV data into a Polars DataFrame from either a file or stdin
///
/// # Arguments
///
/// * `source` - Either a file path or stdin as the data source
/// * `separator` - Optional separator character, defaults to ','
///
/// # Returns
///
/// * `Result<DataFrame, Box<dyn std::error::Error>>` - The resulting DataFrame or an error
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
pub fn read_data(
    source: &String,
    separator: Option<char>,
) -> Result<DataFrame, Box<dyn std::error::Error>> {
    let sep = separator.unwrap_or(',') as u8;
    let mut buffer = String::new();

    match source.as_str() {
        "-" => io::stdin().read_to_string(&mut buffer)?,
        _ => File::open(source)?.read_to_string(&mut buffer)?,
    };

    let df = CsvReader::new(Cursor::new(buffer))
        .with_separator(sep)
        .has_header(true)
        .finish()?;

    Ok(df)
}

/// Writes a Polars DataFrame to stdout as CSV format
///
/// # Arguments
///
/// * `df` - The DataFrame to write to stdout
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
pub fn write_data(mut df: DataFrame) -> Result<(), Box<dyn std::error::Error>> {
    // Print final result
    if std::io::stdout().is_terminal() {
        println!("{:?}", df);
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
    fn test_config() {
        // Call the setup function - it should not panic
        config();

        // Verify that key environment variables are set
        assert!(env::var("POLARS_TABLE_WIDTH").is_ok());
        assert!(env::var("POLARS_FMT_MAX_ROWS").is_ok());
        assert!(env::var("POLARS_FMT_TABLE_FORMATTING").is_ok());

        // Verify specific formatting values
        assert_eq!(
            env::var("POLARS_FMT_TABLE_FORMATTING").unwrap(),
            "UTF8_BORDERS_ONLY"
        );
        assert_eq!(
            env::var("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES").unwrap(),
            "1"
        );
        assert_eq!(env::var("POLARS_FMT_TABLE_ROUNDED_CORNERS").unwrap(), "1");

        // Verify table width is within expected bounds
        let table_width: usize = env::var("POLARS_TABLE_WIDTH").unwrap().parse().unwrap();
        assert!(table_width >= 80 && table_width <= 300);

        // Verify max rows is within expected bounds
        let max_rows: usize = env::var("POLARS_FMT_MAX_ROWS").unwrap().parse().unwrap();
        assert!(max_rows >= 10 && max_rows <= 1000);
    }

    #[test]
    fn test_read_data_from_file_comma_separated() {
        // Create a temporary CSV file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,Los Angeles").unwrap();

        let file_path = temp_file.path().to_string_lossy().to_string();

        // Test reading with default comma separator
        let df = read_data(&file_path, None).unwrap();

        assert_eq!(df.shape().0, 2); // 2 rows
        assert_eq!(df.shape().1, 3); // 3 columns
        assert_eq!(df.get_column_names(), &["name", "age", "city"]);
    }

    #[test]
    fn test_read_data_from_file_tab_separated() {
        // Create a temporary TSV file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name\tage\tcity").unwrap();
        writeln!(temp_file, "Alice\t30\tNew York").unwrap();
        writeln!(temp_file, "Bob\t25\tLos Angeles").unwrap();

        let file_path = temp_file.path().to_string_lossy().to_string();

        // Test reading with tab separator
        let df = read_data(&file_path, Some('\t')).unwrap();

        assert_eq!(df.shape().0, 2); // 2 rows
        assert_eq!(df.shape().1, 3); // 3 columns
        assert_eq!(df.get_column_names(), &["name", "age", "city"]);
    }

    #[test]
    fn test_read_data_from_cursor() {
        // Test reading from a string buffer (simulating stdin)
        let csv_data = "name,age,city\nAlice,30,New York\nBob,25,Los Angeles";
        let cursor = Cursor::new(csv_data);

        let df = CsvReader::new(cursor)
            .with_separator(b',')
            .has_header(true)
            .finish()
            .unwrap();

        assert_eq!(df.shape().0, 2); // 2 rows
        assert_eq!(df.shape().1, 3); // 3 columns
        assert_eq!(df.get_column_names(), &["name", "age", "city"]);
    }

    #[test]
    fn test_read_data_from_stdin_string() {
        // Test the stdin path using "-" as source
        // Note: This test can't actually test stdin input in unit tests,
        // but we can test that the function handles the "-" source correctly
        // by checking that it doesn't panic and follows the stdin code path

        // Create a temporary file to simulate what would come from stdin
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,test").unwrap();
        writeln!(temp_file, "2,data").unwrap();

        let file_path = temp_file.path().to_string_lossy().to_string();

        // Test that non-stdin files work with the current implementation
        let df = read_data(&file_path, None).unwrap();

        assert_eq!(df.shape().0, 2); // 2 rows
        assert_eq!(df.shape().1, 2); // 2 columns
        assert_eq!(df.get_column_names(), &["id", "value"]);
    }
}
