try:
  cargo run -- ~/Downloads/Takeout ~/Documents/tmp

check:
  cargo clippy
  cargo test

newdep crate_name:
  cargo add {{crate_name}}

sync:
  git checkout main
  git pull
