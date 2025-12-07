# project justfile

import? '.just/shellcheck.just'
import? '.just/compliance.just'
import? '.just/gh-process.just'

# just list (default)
list_recipes:
  just --list

# run the code and see how it goes (default)
[group('Rust')]
try:
  cargo run -- examples test_output

# run with backtrace enabled
[group('Rust')]
backtrace:
  RUST_BACKTRACE=1 cargo run -- examples test_output

# what have you broken?
[group('Rust')]
check:
  cargo fmt --check
  cargo check
  cargo clippy
  cargo test --workspace
  cargo audit

# add a crate dependancy
[group('Rust')]
newdep crate_name:
  cargo add {{crate_name}}
  cargo doc
