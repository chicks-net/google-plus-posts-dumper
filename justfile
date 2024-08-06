try:
  cargo run -- ~/Downloads/Takeout ~/Documents/tmp

backtrace:
  RUST_BACKTRACE=1 cargo run -- ~/Downloads/Takeout ~/Documents/tmp

check:
  cargo clippy
  cargo test

newdep crate_name:
  cargo add {{crate_name}}
  cargo doc

sync:
  git checkout main
  git pull
  cargo doc
