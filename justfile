try:
  cargo run -- ~/Downloads/Takeout

check:
  cargo clippy

sync:
  git checkout main
  git pull
