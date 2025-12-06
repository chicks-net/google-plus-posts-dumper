# project justfile

import? '.just/shellcheck.just'
import? '.just/compliance.just'
import? '.just/gh-process.just'

# just list (default)
list_recipes:
  just --list

# run the code and see how it goes (default)
try:
  cargo run -- examples test_output

# run with backtrace enabled
backtrace:
  RUST_BACKTRACE=1 cargo run -- examples test_output

# what have you broken?
check:
  cargo check
  cargo clippy
  cargo test --workspace

# add a crate dependancy
newdep crate_name:
  cargo add {{crate_name}}
  cargo doc
