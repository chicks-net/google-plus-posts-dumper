# run the code and see how it goes (default)
try:
  cargo run -- ~/Downloads/Takeout ~/Documents/tmp

# run with backtrace enabled
backtrace:
  RUST_BACKTRACE=1 cargo run -- ~/Downloads/Takeout ~/Documents/tmp

# what have you broken?
check:
  cargo clippy
  cargo test

# add a crate dependancy
newdep crate_name:
  cargo add {{crate_name}}
  cargo doc

# get back to a clean start
sync:
  git checkout main
  git pull
  cargo doc
