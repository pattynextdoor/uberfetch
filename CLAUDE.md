# uberfetch

## Pre-commit checks

Before every commit, run these checks and ensure they all pass:

```sh
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
```

If `cargo fmt --check` fails, run `cargo fmt` to fix formatting before committing. Do not commit code that fails any of these checks.
