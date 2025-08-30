# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust command-line tool that parses HTML files from Google+ data dumps and converts them to Hugo-friendly Markdown. The project is in development and currently stops with a panic after parsing HTML structure.

## Architecture

- **Single-file application**: `src/main.rs` contains all functionality
- **HTML parsing**: Uses `html5ever` and `markup5ever_rcdom` for DOM parsing
- **File handling**: Uses `glob` crate for pattern matching HTML files
- **Structure**: 
  - `main()`: Argument parsing, directory validation, file discovery
  - `assert_dir()`: Directory existence validation
  - `process_file()`: HTML parsing and processing (incomplete)
  - `walk()`: Recursive DOM traversal for debugging

## Development Commands

### Build and Test
```bash
# Run linting and tests
just check
# Equivalent to:
cargo clippy
cargo test --workspace
```

### Running the Application
```bash
# Default run with hardcoded paths
just

# Run with custom source and destination
cargo run -- <google_plus_dump_dir> <markdown_dest_dir>

# Run with backtrace enabled
just backtrace
```

### Development Workflow
```bash
# Add new dependency
just newdep <crate_name>

# Return to clean main branch
just sync

# Create PR from current branch
just pr

# Merge PR and return to main
just merge
```

## Expected Directory Structure

The tool supports two input structures:
- **Google+ Takeout**: Directory containing `Google+ Stream/Posts/*.html`
- **Direct HTML directory**: Any directory containing `.html` files (like the `examples/` directory)
- **Output**: Destination directory for generated Markdown files

## Generated Markdown Structure

Each converted post includes:
- **Metadata**: Author, date, location, sharing visibility
- **Main content**: Post text with preserved formatting
- **Media sections**: Images, videos, embedded links
- **Social activity**: +1s from other users
- **Comments**: Full comment threads with authors and timestamps

## Current State

- ✅ Command-line argument parsing
- ✅ Directory validation
- ✅ HTML file discovery via glob patterns
- ✅ HTML parsing with `html5ever`
- ✅ Structured data extraction from Google+ posts
- ✅ Markdown generation with full post content
- ✅ Support for multiple post types (text, images, videos, links, location)
- ✅ Comment parsing and formatting
- ✅ Social activity parsing (+1s, visibility)

The application now successfully converts Google+ HTML files to structured Markdown files.

## Testing

Tests are configured but the current doctest example may not work properly with `cargo test`. The CI runs `just check` which includes `cargo test --workspace`.