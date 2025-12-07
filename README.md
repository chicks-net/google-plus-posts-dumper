# google-plus-posts-dumper

[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
![GitHub License](https://img.shields.io/github/license/chicks-net/google-plus-posts-dumper)
![GitHub repo size](https://img.shields.io/github/repo-size/chicks-net/google-plus-posts-dumper)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/chicks-net/google-plus-posts-dumper/verify.yaml)

## Motivation

- I wanted to do something in rust since it has a reputation for being hard.
- I had a pile of HTML from my google+ to convert to hugo-friendly Markdown.

## Status

It does everything I have imagined it wanting to do.  I'm working on verifying
the output with hugo.

## Disclaimer and eating my words

Be careful about saying stuff like:

> This code was written by human hands using `vim` without performance
> enhancing drugs or AI coding assistance.  If you're into those things,
> you be you.  But at this point in my life I'd rather see how well I
> can do naturally.  I feel confidant that I'm learning more this way.

because you might eventually get lazy enough to give into the AI
as a way to get things done.  C'est la vie.  Never say never.

At least I'm still editing with `vim`.

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
stanza which processes the `examples/` directory and outputs to `test_output/`.

The project includes:

- `examples/` - Sample Google+ HTML files for testing the parser
- `test_output/` - Generated Markdown files (gitignored)

Other named recipes are:

- `check` - run rust linters locally
- `newdep crate_name` - add a new create dependancy
- `sync` - get out of a branch after merging
- `backtrace` - run with detailed error traces

So `just check` would rerun the linters for you.

I'm still new to `just`, but it has been helpful while developing this
project.  I'm working on a blog post or youtube video about my happy
experience with `just`.
