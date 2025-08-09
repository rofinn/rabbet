//! CLI Integration Tests using trycmd
//!
//! This module uses trycmd to test CLI commands by running them and comparing
//! their output against expected snapshots. This ensures that the examples in
//! our README.md and other documentation remain accurate and functional.
//!
//! ## Test Cases
//!
//! - `README.md`: Tests the examples in the README, specifically the join and query commands
//! - `examples/cmd/*.toml`: TOML-based test cases for more complex scenarios
//! - `examples/cmd/*.trycmd`: Markdown-style test cases (like help commands)
//!
//! ## Running Tests
//!
//! - `cargo test --test cli_tests` - Run all CLI tests
//! - `TRYCMD=dump cargo test --test cli_tests` - Generate snapshots for new tests
//! - `TRYCMD=overwrite cargo test --test cli_tests` - Update existing snapshots
//!
//! ## Format Override
//!
//! The CLI commands in tests use the default `--format auto` behavior, which is
//! overridden by environment variables to produce table output even when stdout
//! is not a terminal. This makes test commands look exactly like what users would
//! type in real terminals.
//!
//! ## Environment Variables
//!
//! - `POLARS_TABLE_WIDTH=220`: Set globally for all tests to ensure consistent
//!   table formatting with improved readability. This provides wider tables with
//!   full column names instead of truncated versions.
//! - `RABBET_FORCE_TABLE=1`: Forces table output when using `--format auto` (the
//!   default), simulating terminal behavior. This eliminates the need for explicit
//!   `--format table` flags in test commands, making them cleaner and more realistic.

#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .env("POLARS_TABLE_WIDTH", "220")
        .env("RABBET_TABLE_OUTPUT", "1")
        .case("README.md")
        .case("examples/**/*.toml")
        .case("examples/**/*.trycmd");
}
