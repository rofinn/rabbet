# Rabbet

<p align="center">
    <img src="./docs/rabbet.svg" alt="Rabbet Joint Diagram" width="400" />
</p>

Simple user-friendly joins on the command line.

## Installation

### From Source

```bash
git clone https://github.com/rofinn/rabbet.git
cd rabbet
cargo install --path .
```

## Quick Start

Join two CSV files on matching columns:

```bash
rabbet tests/data/customers.csv tests/data/orders.csv --on id,customer_id
```

This performs an inner join between `tests/data/customers.csv` and `tests/data/orders.csv` where the `id` column in the first table matches the `customer_id` column in the second table.

Expected output (formatted as a markdown table):
```
| id | name | email | city | region | country | join_date | order_id | amount | status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | John Smith | john.smith@example.com | New York | NY | USA | 2023-01-15 | ORD-001 | 125.99 | Delivered |
| 1 | John Smith | john.smith@example.com | New York | NY | USA | 2023-01-15 | ORD-003 | 45.25 | Delivered |
| 2 | Emma Johnson | emma.j@example.com | London | England | UK | 2023-02-20 | ORD-002 | 89.50 | Delivered |
| 3 | Miguel Santos | miguel.s@example.com | Madrid | Madrid | Spain | 2023-03-10 | ORD-004 | 220.00 | Shipped |
```

Expected output (in markdown table format):
```
| id | name | email | order_id | customer_id | amount |
| --- | --- | --- | --- | --- | --- |
| 1 | John Smith | john@example.com | 101 | 1 | 125.99 |
| 1 | John Smith | john@example.com | 103 | 1 | 45.25 |
| 2 | Jane Doe | jane@example.com | 102 | 2 | 89.50 |
```

## Usage Examples

### Basic Inner Join

```bash
rabbet /path/to/customers.csv /path/to/orders.csv --on id,customer_id
```

Expected output (markdown table format):
```
| id | name | email | order_id | customer_id | amount |
| --- | --- | --- | --- | --- | --- |
| 1 | John Smith | john@example.com | 101 | 1 | 125.99 |
| 1 | John Smith | john@example.com | 103 | 1 | 45.25 |
| 2 | Jane Doe | jane@example.com | 102 | 2 | 89.50 |
```

### Left Join

Include all rows from the left table, even without matches:

```bash
rabbet /path/to/customers.csv /path/to/orders.csv --on id,customer_id --type left
```

Expected output (markdown table format):
```
| id | name | email | order_id | customer_id | amount |
| --- | --- | --- | --- | --- | --- |
| 1 | John Smith | john@example.com | 101 | 1 | 125.99 |
| 1 | John Smith | john@example.com | 103 | 1 | 45.25 |
| 2 | Jane Doe | jane@example.com | 102 | 2 | 89.50 |
| 3 | Robert Jones | robert@example.com |  |  |  |
```

### Output as CSV

```bash
rabbet /path/to/customers.csv /path/to/orders.csv --on id,customer_id --fmt csv > /path/to/joined.csv
```

Contents of `/path/to/joined.csv`:
```
id,name,email,order_id,customer_id,amount
1,John Smith,john@example.com,101,1,125.99
1,John Smith,john@example.com,103,1,45.25
2,Jane Doe,jane@example.com,102,2,89.50
```

### Reading from stdin

You can use `-` to read from stdin:

```bash
cat /path/to/customers.csv | rabbet - /path/to/orders.csv --on id,customer_id
```

Output is the same as the basic inner join example.

### Joining Multiple Tables

Join three tables in sequence:

```bash
rabbet /path/to/customers.csv /path/to/orders.csv /path/to/items.csv --on id,customer_id,order_id,order_id
```

Expected output (abbreviated):
```
| id | name | ... | order_id | ... | item_id | product |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | John Smith | ... | 101 | ... | 201 | Laptop |
| 1 | John Smith | ... | 101 | ... | 202 | Mouse |
| 2 | Jane Doe | ... | 102 | ... | 203 | Keyboard |
```

### Working with TSV Files

```bash
rabbet /path/to/data.tsv /path/to/other_data.tsv --on id --delimiter $'\t'
```

Expected output (assuming TSV files have matching ids):
```
| id | name | value | category | active |
| --- | --- | --- | --- | --- |
| 1 | Alice | 100 | A | true |
| 2 | Bob | 200 |

## Join Types

Rabbet supports the following join types:

| Type | Description |
|------|-------------|
| `inner` (default) | Returns rows when there is a match in both tables |
| `left` | Returns all rows from the left table and matching rows from the right table |
| `right` | Returns all rows from the right table and matching rows from the left table |
| `outer` | Returns all rows when there is a match in either left or right table |
| `cross` | Returns the Cartesian product of both tables |

## CLI Arguments

```
USAGE:
    rabbet [OPTIONS] <TABLES>...

ARGS:
    <TABLES>...    Input tables (files or '-' for stdin)

OPTIONS:
    --on <COLUMNS>        Columns to join on (comma separated)
    --type <TYPE>         Type of join to perform [default: inner] [possible values: inner, left, right, outer, cross]
    --fmt <FORMAT>        Output format [default: markdown] [possible values: markdown, csv, tsv]
    --delimiter <CHAR>    Delimiter for input files [default: ,]
    -h, --help            Print help information
    -V, --version         Print version information
```

### Join Column Specification

The `--on` argument accepts a comma-separated list of column names:

- With a single value (`--on id`): Joins tables on columns with this name in all tables
- With two values (`--on id,customer_id`): Joins where the first table's `id` matches the second table's `customer_id`
- With multiple values: For multi-table joins, column pairs alternate between tables (left1,right1,left2,right2...)

For cross joins, the `--on` parameter is not required.

## Examples with Actual Test Data

Our repository contains test data you can use to try out the tool. Let's look at some examples using the actual test files.

### Sample Test Data

**tests/data/customers.csv:**
```csv
id,name,email,city,region,country,join_date
1,John Smith,john.smith@example.com,New York,NY,USA,2023-01-15
2,Emma Johnson,emma.j@example.com,London,England,UK,2023-02-20
3,Miguel Santos,miguel.s@example.com,Madrid,Madrid,Spain,2023-03-10
4,Sophie Martin,sophie.m@example.com,Paris,Île-de-France,France,2023-04-05
5,Li Wei,li.wei@example.com,Beijing,Beijing,China,2023-05-12
6,Mary Williams,mary.w@example.com,Sydney,NSW,Australia,2023-06-18
```

**tests/data/orders.csv:**
```csv
id,order_id,customer_id,order_date,amount,status
1,ORD-001,1,2023-02-01,125.99,Delivered
2,ORD-002,2,2023-02-15,89.50,Delivered
3,ORD-003,1,2023-03-10,45.25,Delivered
4,ORD-004,3,2023-04-05,220.00,Shipped
5,ORD-005,2,2023-04-22,65.75,Shipped
6,ORD-006,4,2023-05-15,175.25,Processing
7,ORD-007,1,2023-06-10,95.00,Processing
8,ORD-008,7,2023-06-20,150.50,Processing
```

### Inner Join Example with Test Data

Command:
```bash
rabbet tests/data/customers.csv tests/data/orders.csv --on id,customer_id --fmt csv
```

Output:
```csv
id,name,email,city,region,country,join_date,id,order_id,order_date,amount,status
1,John Smith,john.smith@example.com,New York,NY,USA,2023-01-15,1,ORD-001,2023-02-01,125.99,Delivered
1,John Smith,john.smith@example.com,New York,NY,USA,2023-01-15,3,ORD-003,2023-03-10,45.25,Delivered
1,John Smith,john.smith@example.com,New York,NY,USA,2023-01-15,7,ORD-007,2023-06-10,95.00,Processing
2,Emma Johnson,emma.j@example.com,London,England,UK,2023-02-20,2,ORD-002,2023-02-15,89.50,Delivered
2,Emma Johnson,emma.j@example.com,London,England,UK,2023-02-20,5,ORD-005,2023-04-22,65.75,Shipped
3,Miguel Santos,miguel.s@example.com,Madrid,Madrid,Spain,2023-03-10,4,ORD-004,2023-04-05,220.00,Shipped
4,Sophie Martin,sophie.m@example.com,Paris,Île-de-France,France,2023-04-05,6,ORD-006,2023-05-15,175.25,Processing
```

### Left Join Example with Test Data

Command:
```bash
rabbet tests/data/customers.csv tests/data/orders.csv --on id,customer_id --type left --fmt csv
```

Output:
```csv
id,name,email,city,region,country,join_date,id,order_id,order_date,amount,status
1,John Smith,john.smith@example.com,New York,NY,USA,2023-01-15,1,ORD-001,2023-02-01,125.99,Delivered
1,John Smith,john.smith@example.com,New York,NY,USA,2023-01-15,3,ORD-003,2023-03-10,45.25,Delivered
1,John Smith,john.smith@example.com,New York,NY,USA,2023-01-15,7,ORD-007,2023-06-10,95.00,Processing
2,Emma Johnson,emma.j@example.com,London,England,UK,2023-02-20,2,ORD-002,2023-02-15,89.50,Delivered
2,Emma Johnson,emma.j@example.com,London,England,UK,2023-02-20,5,ORD-005,2023-04-22,65.75,Shipped
3,Miguel Santos,miguel.s@example.com,Madrid,Madrid,Spain,2023-03-10,4,ORD-004,2023-04-05,220.00,Shipped
4,Sophie Martin,sophie.m@example.com,Paris,Île-de-France,France,2023-04-05,6,ORD-006,2023-05-15,175.25,Processing
5,Li Wei,li.wei@example.com,Beijing,Beijing,China,2023-05-12,,,,,,
6,Mary Williams,mary.w@example.com,Sydney,NSW,Australia,2023-06-18,,,,,,
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_inner_join
```

### Project Structure

- `src/args.rs` - Command-line argument parsing and validation
- `src/tables.rs` - Table structure, reading, and formatting
- `src/join.rs` - Hash-based join implementations
- `src/main.rs` - CLI entry point

## License

MIT or Apache-2.0, at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request
