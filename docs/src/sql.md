# SQL

The `query` command allows you to run SQL queries against CSV, JSON, and Parquet files as if they were database tables.

## Basic Usage

```bash
rabbet query --as <table_name> <file> -- "<SQL query>"
```

## Arguments

- `--as`: Alias name for the table in your SQL query
- `file`: Input file (CSV, JSON, or Parquet)
- `--`: Separator between options and the SQL query
- `SQL query`: Standard SQL SELECT statement

## Multiple Tables

You can query multiple files by specifying multiple `--as` options:

```bash
rabbet query --as customers customers.csv --as orders orders.csv -- \
  "SELECT * FROM customers JOIN orders ON customers.id = orders.customer_id"
```

## Supported SQL Features

- `SELECT` with column selection and aliases
- `FROM` with table aliases
- `WHERE` with complex conditions
- `JOIN` (INNER, LEFT, RIGHT, FULL)
- `GROUP BY` with aggregations
- `ORDER BY` with ASC/DESC
- `LIMIT` and `OFFSET`
- Common functions (COUNT, SUM, AVG, MIN, MAX, etc.)
- String functions (UPPER, LOWER, CONCAT, etc.)
- Date functions (DATE_TRUNC, EXTRACT, etc.)

## Examples

### Basic Query

{{#include ../../examples/query/simple-query.trycmd}}

### Aggregation Query

{{#include ../../examples/query/aggregation-query.trycmd}}

### Complex Join Query

For complex multi-table queries:

```bash
rabbet query --as customers data/orders/customers.csv --as orders data/orders/orders.csv -- <<EOF
SELECT
    c.customer_name,
    c.customer_city,
    COUNT(o.order_id) as order_count,
    SUM(o.quantity * o.price) as total_spent,
FROM customers c
LEFT JOIN orders o ON c.customer_id = o.customer_id
GROUP BY c.customer_name, c.customer_city
HAVING order_count > 0
ORDER BY total_spent DESC
EOF
╭─────────────────────────────────────────────────────────────╮
│ customer_name     customer_city   order_count   total_spent │
╞═════════════════════════════════════════════════════════════╡
│ Robert Brown      Anytown         1             250.0       │
│ Emily Davis       Anytown         1             160.0       │
│ Michael Johnson   Anytown         3             140.0       │
╰─────────────────────────────────────────────────────────────╯

```


## Notes

- Table names in SQL must match the aliases specified with `--as`
- SQL keywords are case-insensitive
- Column names are case-sensitive and must match the file headers
- The query must be a SELECT statement (no INSERT, UPDATE, DELETE)
- Complex queries may require more memory for processing
