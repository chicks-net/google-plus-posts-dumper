# project justfile

import? '.just/compliance.just'
import? '.just/gh-process.just'

# run the code and see how it goes (default)
try:
  cargo run -- examples test_output

# run with backtrace enabled
backtrace:
  RUST_BACKTRACE=1 cargo run -- examples test_output

# what have you broken?
check:
  cargo clippy
  cargo test --workspace

# add a crate dependancy
newdep crate_name:
  cargo add {{crate_name}}
  cargo doc
