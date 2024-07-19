try:
  cargo run -- ~/Downloads/Takeout ~/Documents/tmp

check:
  cargo clippy
  cargo test

sync:
  git checkout main
  git pull
