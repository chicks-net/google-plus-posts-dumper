# run the code and see how it goes (default)
try:
  cargo run -- ~/Downloads/Takeout ~/Documents/tmp

# run with backtrace enabled
backtrace:
  RUST_BACKTRACE=1 cargo run -- ~/Downloads/Takeout ~/Documents/tmp

# what have you broken?
check:
  cargo clippy
  cargo test --workspace

# add a crate dependancy
newdep crate_name:
  cargo add {{crate_name}}
  cargo doc

# get back to a clean start
sync:
  git checkout main
  git pull
  cargo doc

release_branch := "main"

# error if not on a git branch
on_a_branch:
  #!/bin/bash

  # thanks to https://stackoverflow.com/a/12142066/2002471

  if [[ $(git rev-parse --abbrev-ref HEAD) == "{{release_branch}}" ]]; then
    echo "You are on branch {{release_branch}} (the release branch) so you are not ready to start a PR."
    exit 100
  fi

# thanks to https://stackoverflow.com/a/7293026/2002471 for the perfect git incantation
last_commit_message := `git log -1 --pretty=%B | grep .`
pr_tmpfile := '/tmp/just-pr-body.txt'

# PR create v2.0
pr: on_a_branch
  git stp
  git pushup
  #gh pr create --fill-verbose
  #gh pr create --title "{{last_commit_message}}" --body "{{last_commit_message}} (Automated in justfile.)"
  echo "Did: {{last_commit_message}}\n\n(Automated in justfile.)\n" > {{ pr_tmpfile }}
  gh pr create --title "{{last_commit_message}}" -F  {{ pr_tmpfile }}
  rm {{ pr_tmpfile }}

# PR merge and return to main branch
merge: on_a_branch
  gh pr merge -s
  git co {{release_branch}}
  git pull
