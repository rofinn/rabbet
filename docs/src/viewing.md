# Viewing

Rabbet provides several commands for viewing and inspecting data files in a formatted table view. These commands are useful for quickly examining your data without performing any transformations.

## Commands Overview

- `cat` - Display the entire contents of a file
- `head` - Display the first N rows of a file
- `tail` - Display the last N rows of a file

## Basic Usage

```bash
rabbet cat <file>
rabbet head <file> [-n <number>]
rabbet tail <file> [-n <number>]
```

## Common Options

All viewing commands support these options:
- `--format`: Output format - `table` (default), `csv`, `json`, or `jsonl`
- `--delimiter`: Input file delimiter (default: `,`)
- `--no-header`: Treat the first row as data instead of column headers

## Examples

### Viewing Entire Files with `cat`

Display all rows from a CSV file in a formatted table:

{{#include ../../examples/cat/basic.trycmd}}

### Viewing First Rows with `head`

Display the first 3 rows of a file:

{{#include ../../examples/head/basic.trycmd}}

### Viewing Last Rows with `tail`

Display the last 3 rows of a file:

{{#include ../../examples/tail/basic.trycmd}}

## Use Cases

- **Quick inspection**: Use `cat` for small files to see all data at once
- **Preview large files**: Use `head` to check the structure and first few rows
- **Check recent entries**: Use `tail` to see the most recent records in time-series or log data
- **Verify headers**: Use `head -n 1` to quickly check column names
- **Data validation**: Combine with other formats (`--format csv`) to verify parsing

## Notes

- The default number of rows for `head` and `tail` is 10 when `-n` is not specified
- Table format automatically truncates long values for display
- For very wide tables, consider using `--format csv` for better readability
- These commands preserve the original data types and formatting