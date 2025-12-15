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

# count posts per month
[group('Rust')]
count_posts output_dir='test_output':
  #!/usr/bin/env bash
  set -euo pipefail
  if [ ! -d "{{output_dir}}" ]; then
    echo "Directory {{output_dir}} does not exist"
    exit 1
  fi
  echo "Post count by month in {{output_dir}}:"
  echo "======================================"
  find "{{output_dir}}" -name "*.md" -maxdepth 1 2>/dev/null | \
    sed 's|.*/||' | \
    grep -E '^[0-9]{4}-[0-9]{2}-' | \
    cut -d'-' -f1,2 | \
    sort | \
    uniq -c | \
    awk '{printf "%s-%s: %2d posts\n", substr($2,1,4), substr($2,6,2), $1}' || \
    echo "No markdown files found"
