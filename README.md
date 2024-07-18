# google-plus-posts-dumper

[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)

## Motivation

* I wanted to do something in rust.
* I had a pile of HTML from my google+ dump that needs to be converted to hugo-friendly Markdown.

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

```
cargo run -- $GOOGLE_PLUS_DUMP_DIR $MARKDOWN_DEST_DIR
```

It takes two arguments:

1. The directory of the Google+ dump aka "Takeout".  It needs to contain the
`Google+ Stream/Posts` directory structure.
1. The directory where you want the Markdown files created.

You can see in the `justfile` how I've been testing this on my machine.
