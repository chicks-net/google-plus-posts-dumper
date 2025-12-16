# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust command-line tool that parses Google+ Takeout HTML files and converts them to Hugo-friendly Markdown with full post content, metadata, comments, and social activity.

## Architecture

**Single-file monolith**: All functionality lives in `src/main.rs` (~650 lines)

**Data flow**:

1. `main()` - CLI argument parsing, directory validation, HTML file discovery via `glob`
2. `process_file()` - Per-file orchestration: parse HTML → extract data → generate Markdown → write file
3. `extract_post_data()` / `find_post_elements()` - Recursive DOM traversal extracting structured data into `PostData`
4. `generate_markdown()` - Convert `PostData` to Hugo-compatible Markdown with TOML frontmatter

**Key structures**:

- `PostData` - Complete post representation: author, date, content, reshares, location, media, comments, +1s
- `Comment` - Comment thread data with author/date/content

**HTML parsing**: Uses `html5ever` + `markup5ever_rcdom` for DOM traversal with CSS class-based element identification

**Domain logic**:

- `convert_to_utc()` - Parse Google+ timestamps (`YYYY-MM-DD HH:MM:SS±HHMM`) to ISO 8601 UTC
- `format_filename_date()` - Transform `20110814 - Title.html` to `2011-08-14-Title.md`
- `escape_toml_string()` - Quote/backslash escaping for TOML frontmatter
- Helper functions for DOM queries: `has_class()`, `get_attr_value()`, `get_text_content()`, etc.

## Development Commands

**Build/test/lint**:

```bash
just check          # Run all checks (cargo fmt --check + cargo check + clippy + test + audit)
cargo clippy        # Linting only
cargo test          # Tests only (currently has doctest issues)
cargo audit         # Security audit of dependencies
```

Claude should run "just check" instead of "cargo test".

**Running**:

```bash
just                # Default: process examples/ → test_output/
just try            # Same as above
just backtrace      # Run with RUST_BACKTRACE=1
cargo run -- <source_dir> <dest_dir>  # Custom paths
```

**Dependencies**:

```bash
just newdep <crate_name>  # Add dependency + regenerate docs
```

**Git/GitHub workflow** (via `.just/gh-process.just`):

```bash
just branch <name>  # Create new branch with user/$DATE-name format
just pr             # Push + create PR with auto-generated description
just pr_checks      # Watch CI + check for Copilot/Claude suggestions
just pr_update      # Update PR description's "Done" section with current commits
just merge          # Squash-merge PR, delete branch, return to main
just sync           # Return to main + pull latest
```

## Input/Output Structure

**Input** (two supported formats):

1. Google+ Takeout: `<dump_dir>/Google+ Stream/Posts/*.html`
2. Direct HTML: `<any_dir>/*.html` (e.g., `examples/`)

**Output**: `<dest_dir>/<YYYY-MM-DD-filename>.md`

**Generated Markdown includes**:

- TOML frontmatter (title, date, description, tags, cover settings)
- Metadata line (author, location, visibility)
- Main post content with preserved formatting
- Reshared content attribution
- Media sections (images, videos, links)
- Social activity (+1s)
- Full comment threads

## CI/CD

GitHub Actions (`.github/workflows/verify.yaml`) runs `just check` on all PRs and main branch pushes across macOS (14 + latest), Windows, Ubuntu with `RUSTFLAGS=--deny warnings`. CI includes `cargo-audit` for dependency security scanning.

## Testing

**Unit tests**: Located in `src/main.rs:725` in a `#[cfg(test)]` module with ~55 tests covering:

- `escape_toml_string()` - TOML string escaping and newline handling
- `convert_to_utc()` - Timestamp parsing and timezone conversion
- `format_filename_date()` - Filename transformation from Google+ format to Hugo format
- `clean_title()` - HTML entity decoding and tag stripping

**Integration tests**: The `examples/` directory contains sample Google+ HTML files used for manual testing with `just try`

## Current Limitations

- Doctest in `src/main.rs:10` doesn't execute with `cargo test` (known issue, documented in code)
