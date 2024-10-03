# google-plus-posts-dumper

[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
![GitHub License](https://img.shields.io/github/license/chicks-net/google-plus-posts-dumper)
![GitHub repo size](https://img.shields.io/github/repo-size/chicks-net/google-plus-posts-dumper)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/chicks-net/google-plus-posts-dumper/verify.yaml)
![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fchicks-net%2Fwww-chicks-net%2Frefs%2Fheads%2Fmain%2Fhugo.toml&query=%24.theme%5B0%5D&label=theme%5B0%5D)

## Motivation

* I wanted to do something in rust since it has a reputation for being hard.
* I had a pile of HTML from my google+ to convert to hugo-friendly Markdown.

## Status

It processes command line arguments and verifies that the directories are
there and laid out as expected.  Then it finds the HTML files in the first
directory.

## Disclaimer

This code was written by human hands using `vim` without performance
enhancing drugs or AI coding assistance.  If you're into those things,
you be you.  But at this point in my life I'd rather see how well I
can do naturally.  I feel confidant that I'm learning more this way.

## Documentation

```zsh
cargo run -- $GOOGLE_PLUS_DUMP_DIR $MARKDOWN_DEST_DIR
```

It takes two arguments:

1. The directory of the Google+ dump aka "Takeout".  It needs to contain the
`Google+ Stream/Posts` directory structure.
1. The directory where you want the Markdown files created.

### Be just

Check out [casey/just](https://github.com/casey/just) if you haven't heard of
`just` yet.

You can see in the [`justfile`](./justfile) how I've been using it to build this
project on my machine.  Running `just` without arguments will run the `try`
stanza where I have setup my source and destination directories.  Feel free to
edit this for your own convenience -- you would be `just`ified.  :grin:

Other named recipes are:

* `check` - run rust linters locally
* `newdep crate_name` - add a new create dependancy
* `sync` - get out of a branch after merging

So `just check` would rerun the linters for you.

I'm still new to `just`, but it has been helpful while developing this
project.  I'm working on a blog post or youtube video about my happy
experience with `just`.
