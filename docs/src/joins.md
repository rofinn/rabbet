# Joins

The `join` command combines rows from two tables based on related columns between them, similar to SQL JOIN operations.

## Basic Usage

```bash
rabbet join <left-table> <right-table> --on <column> [options]
```

## Arguments

- `left-table`: First input CSV file
- `right-table`: Second input CSV file  
- `--on`: Column name to join on (must exist in both tables)
- `--type`: Join type - `inner` (default), `left`, `right`, or `full`
- `--left-on`: Column name in left table (when join columns have different names)
- `--right-on`: Column name in right table (when join columns have different names)
- `--format`: Output format - `table` (default) or `csv`

## Join Types

- **Inner Join** (default): Returns only rows with matching values in both tables
- **Left Join**: Returns all rows from the left table, with NULL values for non-matching right rows
- **Right Join**: Returns all rows from the right table, with NULL values for non-matching left rows
- **Full Join**: Returns all rows from both tables, with NULL values where there's no match

## Examples

### Basic Inner Join

Join two tables on a common column:

{{#include ../../examples/join/csv-format.trycmd}}

### Left Join

Include all rows from the left table, even when there's no match in the right table:

{{#include ../../examples/join/left-join.trycmd}}

## Notes

- The join column must have the same data type in both tables
- Column names from both tables are preserved in the output
- If tables have overlapping column names (other than the join column), they will be prefixed to avoid conflicts
- For best performance, ensure your data is sorted by the join column
- Large joins may require significant memory