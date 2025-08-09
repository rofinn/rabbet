# Development

This is a pretty small project and I don't have an overly strong opinion on how code should be structured.
So far, I've maintained a relatively flat directory structure apart from `main.rs` and `args.rs` (for the top level clap struct) most of the other subcommands are isolated to their own files.
Any shared functionality lives in an appropriately named modules at the same level (e.g., `io`).
I tend to keep unit tests inline with the code they test and use `trycmd` in the examples directory for acceptance tests.
For now, the `/tests` directory just contains a `cli_tests.rs` file for running our examples.

Apart from that there is the standard formatting, linting and testing commands provided below.
I've bundled most of these into a `.pre-commit-config.yaml` file if you don't want to remember them all :)

```console
$ pre-commit run
```
or
```console
$ pre-commit install
```

## Commands

Formatting:

```console
$ cargo fmt
```

Linting:
```console
$ cargo clippy --fix --allow-dirty
```

Testing:
Run all tests (unit + integration):

```console
$ cargo test
```

Testing w/ coverage:
```console
$ cargo llvm-cov
```

Run only CLI integration tests:
```console
$ cargo test --test cli_tests
```

Update CLI test snapshots (should be printed on failures):
```console
$ TRYCMD=overwrite cargo test --test cli_tests
```

Benchmarking:

```console
$ ./scripts/benchmark
```
This will just check that the binary size isn't too large and that cmd latency doesn't exceed 0.1s.
Please update this script for new commands.
We're just using `hyperfine` commands inside it.
