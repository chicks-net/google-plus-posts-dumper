# Source Code Layout

This codebase follows Rust's standard module organization with a flat structure
that separates concerns by functionality.

## Module Overview

```text
src/
├── main.rs          # CLI entry point (120 lines)
├── lib.rs           # Library root with module declarations
├── models.rs        # Data structures
├── dom.rs           # DOM manipulation helpers
├── utils.rs         # String formatting utilities
├── parser.rs        # HTML parsing and extraction
└── markdown.rs      # Markdown generation
```

## Module Details

### main.rs

**Purpose**: CLI application entry point

**Contains**:

- `main()` - Argument parsing, directory validation, file discovery
- `assert_dir()` - Directory validation helper
- `process_file()` - Per-file orchestration (parse → extract → generate → write)

**Dependencies**: Uses library exports from `lib.rs`

### lib.rs

**Purpose**: Library root that declares modules and re-exports public API

**Exports**:

- `extract_post_data()` - Main parsing function
- `generate_markdown()` - Main generation function
- `format_filename_date()` - Filename transformation
- `PostData`, `Comment` - Data structures

### models.rs

**Purpose**: Core data structures representing Google+ posts

**Contains**:

- `PostData` - Complete post representation with author, date, content, media,
  comments, etc.
- `Comment` - Comment data with author, date, content

### dom.rs

**Purpose**: DOM traversal and manipulation helpers

**Contains**:

- `has_class()`, `has_attr()` - Attribute checking
- `get_attr_value()` - Attribute extraction
- `get_text_content()` - Plain text extraction
- `get_text_content_formatted()` - Text with Markdown formatting
- `find_parent_href()` - Parent traversal for links
- `format_markdown_link()` - Convert links to Markdown syntax

**Tests**: 8 tests for link formatting

### utils.rs

**Purpose**: String formatting and transformation utilities

**Contains**:

- `escape_toml_string()` - TOML frontmatter escaping
- `clean_title()` - HTML entity decoding and tag stripping
- `clean_location()` - Location string formatting
- `convert_to_utc()` - Timestamp conversion (Google+ format → ISO 8601 UTC)
- `format_filename_date()` - Filename transformation (YYYYMMDD → YYYY-MM-DD)

**Tests**: 55 tests covering edge cases for all utilities

### parser.rs

**Purpose**: HTML parsing and data extraction from Google+ Takeout files

**Contains**:

- `extract_post_data()` - Main entry point for extraction
- `find_post_elements()` - Recursive DOM traversal
- `extract_comment()` - Comment extraction
- `extract_reshare_content()`, `extract_reshare_text()` - Reshare handling

**Dependencies**: Uses `dom.rs` helpers and `utils.rs` converters

### markdown.rs

**Purpose**: Generate Hugo-compatible Markdown from PostData

**Contains**:

- `generate_markdown()` - Main generation function that produces TOML
  frontmatter and formatted content

**Dependencies**: Uses `utils.rs` for escaping and cleaning

## Data Flow

```text
HTML File (Google+ Takeout)
    ↓
main.rs: process_file()
    ↓
parser.rs: extract_post_data()
    ├→ dom.rs: get_text_content(), has_class(), etc.
    └→ utils.rs: convert_to_utc(), clean_location()
    ↓
models.rs: PostData
    ↓
markdown.rs: generate_markdown()
    └→ utils.rs: escape_toml_string(), clean_title()
    ↓
Markdown File (Hugo-compatible)
```

## Design Principles

**Separation of Concerns**: Each module has a single, clear responsibility

**Flat Structure**: Simple organization appropriate for current codebase size
(can be refactored to hierarchical if needed)

**Library-First**: Core logic lives in library modules, making it testable and
potentially reusable

**Comprehensive Testing**: 71 tests cover edge cases for all utility functions

**Conventional Rust**: Follows standard Rust project layout with
`lib.rs`/`main.rs` separation
