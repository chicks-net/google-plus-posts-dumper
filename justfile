try:
  cargo run -- ~/Downloads/Takeout ~/Documents/tmp

check:
  cargo clippy

sync:
  git checkout main
  git pull
