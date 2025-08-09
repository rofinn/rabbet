# Rabbet

[![CI](https://github.com/rofinn/rabbet/workflows/CI/badge.svg)](https://github.com/rofinn/rabbet/actions?query=workflow%3ACI)
[![codecov](https://codecov.io/gh/rofinn/rabbet/branch/main/graph/badge.svg)](https://codecov.io/gh/rofinn/rabbet)

CLI to simplify shell scripts and glue code operating on relational data.

<p align="center">
    <img src="https://raw.githubusercontent.com/rofinn/rabbet/main/docs/rabbet.svg" alt="Rabbet Joint Diagram" width="400" />
</p>

## Installation

### From Source

```console
$ cargo install --git https://github.com/rofinn/rabbet.git
```

## Quick Start

Join two CSV files on matching columns:

```console
$ rabbet join data/orders/customers.csv data/orders/orders.csv --on customer_id
╭─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮
│ customer_id    customer_name     customer_email      customer_phone   customer_address   customer_city   customer_state   customer_zipcode   customer_country   order_id    product_id    quantity   price   order_date │
╞═════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╡
│ CUSTOMER-003   Michael Johnson   michael.johnson@…   555-9876         789 Oak St         Anytown         CA               90210              USA                ORDER-001   PRODUCT-005   1          10.0    2022-01-01 │
│ CUSTOMER-003   Michael Johnson   michael.johnson@…   555-9876         789 Oak St         Anytown         CA               90210              USA                ORDER-002   PRODUCT-005   2          20.0    2022-01-02 │
│ CUSTOMER-003   Michael Johnson   michael.johnson@…   555-9876         789 Oak St         Anytown         CA               90210              USA                ORDER-003   PRODUCT-003   3          30.0    2022-01-03 │
│ CUSTOMER-004   Emily Davis       emily.davis@exam…   555-2468         321 Pine St        Anytown         CA               90210              USA                ORDER-004   PRODUCT-002   4          40.0    2022-01-04 │
│ CUSTOMER-005   Robert Brown      robert.brown@exa…   555-3698         654 Maple St       Anytown         CA               90210              USA                ORDER-005   PRODUCT-001   5          50.0    2022-01-05 │
╰─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯

```
