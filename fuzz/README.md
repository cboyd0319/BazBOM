# Fuzzing for BazBOM

This directory contains fuzzing tests for BazBOM using `cargo-fuzz`.

## Setup

Install cargo-fuzz:

```bash
cargo install cargo-fuzz
```

## Running Fuzz Tests

### SBOM Parser Fuzzing

```bash
cd fuzz
cargo fuzz run sbom_parser
```

### Dependency Graph Fuzzing

```bash
cargo fuzz run dependency_graph
```

### Policy Engine Fuzzing

```bash
cargo fuzz run policy_engine
```

## Creating New Fuzz Targets

```bash
cargo fuzz add my_target
```

Then edit `fuzz/fuzz_targets/my_target.rs`.

## Continuous Fuzzing

For CI/CD integration:

```bash
# Run for 60 seconds
cargo fuzz run sbom_parser -- -max_total_time=60

# With specific corpus
cargo fuzz run sbom_parser corpus/sbom_parser/
```

## Coverage

Generate coverage report:

```bash
cargo fuzz coverage sbom_parser
```

## References

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer tutorial](https://llvm.org/docs/LibFuzzer.html)
