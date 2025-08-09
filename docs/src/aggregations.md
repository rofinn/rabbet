# Aggregations
The `aggregate` command allows you to group and summarize data using various statistical operations.

## Basic Usage

```bash
rabbet aggregate <table> --by <columns> --with <aggregations>
```

## Arguments

- `table`: Input CSV file or `-` for stdin
- `--by`: Columns to group by (comma-separated)
- `--with`: Aggregation operations as `column=operation` pairs (comma-separated)
- `--delimiter`: Input file delimiter (default: `,`)

## Available Operations

- `sum`: Sum of values
- `mean`: Average of values
- `median`: Median value
- `min`: Minimum value
- `max`: Maximum value
- `range`: Difference between max and min
- `count`: Count of non-null values
- `variance`: Sample variance
- `stddev`: Sample standard deviation
- `first`: First value in group
- `last`: Last value in group
- `describe`: Summary statistics as a string

For row counting operations, use `_=count`, `_=len`, or `_=nrow`.

## Examples

### Simple Aggregation

Group by a single column and calculate one statistic:

{{#include ../../examples/aggregate/simple-aggregation.trycmd}}

### Multiple Aggregations

Calculate multiple statistics for different columns:

{{#include ../../examples/aggregate/multiple-aggregations.trycmd}}

### Multiple Group-By Columns

Group by multiple columns to create finer-grained aggregations:

{{#include ../../examples/aggregate/multiple-groupby.trycmd}}

## Notes

- Column names in the output are automatically suffixed with the operation name (e.g., `PetalLength_mean`)
- When grouping by multiple columns, each unique combination creates a separate group
- Use `--by` without any columns to aggregate the entire dataset into a single row
- Multiple operations can be applied to the same column by specifying it multiple times