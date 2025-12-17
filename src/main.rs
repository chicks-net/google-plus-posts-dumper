//! # google-plus-posts-dumper
//!
//! Parse HTML from posts in google+ data dump
//! to generate hugo-friendly Markdown.
//!
//! Status: in development
//!
//! # Example
//!
//! ```rust
//! assert_dir(Path::new("/"));
//! ```
//!
//! I thought this would work with `cargo test` but maybe I need
//! to jump through more hoops since so far it is ignoring it.
//! I had it fenced with `-----` for `cargo test` but then
//! `cargo doc` didn't format the HTML properly.

extern crate html5ever;
extern crate markup5ever_rcdom as rcdom;

use std::env;
use std::fs::File;
use std::path::Path;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use rcdom::RcDom;

use glob::glob;
use google_plus_posts_dumper::{extract_post_data, format_filename_date, generate_markdown};

fn main() {
    // get directory argument and verify that it is actually a directory
    let args: Vec<String> = env::args().collect();
    // dbg!(args);
    let base_path_arg = args.get(1).expect("Missing required argument: source directory path (e.g., 'examples' or path to Google+ Takeout)");
    let base_path = Path::new(base_path_arg);
    assert_dir(base_path);

    // destination directory
    let dest_path_arg = args.get(2).expect(
        "Missing required argument: destination directory path for generated Markdown files",
    );
    let dest_path = Path::new(dest_path_arg);
    assert_dir(dest_path);

    // find posts directory - either Google+ structure or direct examples
    let posts_path = Path::new(base_path).join("Google+ Stream/Posts");
    let posts_path_string = if posts_path.exists() && posts_path.is_dir() {
        // Original Google+ Takeout structure
        posts_path
            .to_str()
            .expect("Posts path contains invalid UTF-8 characters")
            .to_string()
    } else {
        // Direct examples directory or other structure
        base_path_arg.to_string()
    };
    println!("Posts are in {posts_path_string:?}");

    // Loop through html files
    let mut post_pattern: String = posts_path_string.clone();
    post_pattern.push_str("/*.html");
    println!("Debug: {post_pattern}");
    for entry in glob(&post_pattern).expect("Failed to glob") {
        match entry {
            Ok(path) => process_file(&path.display().to_string(), dest_path_arg),
            Err(e) => println!("{:?}", e),
        }
    }
}

/// Is it a valid directory?
fn assert_dir(dir_path: &Path) {
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());
}

/// Parse an HTML file and generate Markdown
fn process_file(file_name: &str, dest_dir: &str) {
    let file_path = Path::new(file_name);
    assert!(file_path.exists());
    assert!(file_path.is_file());

    println!("processing {:?}", file_name);
    println!("\tinto {:?}", dest_dir);

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file_handle = match File::open(file_path) {
        Err(why) => panic!("couldn't open {}: {}", file_name, why),
        Ok(file_handle) => file_handle,
    };

    // HTML parsing
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut file_handle)
        .unwrap_or_else(|_| panic!("Failed to parse HTML from file: {}", file_name));
    // Parse the document to extract post data
    // Note: html5ever may report parsing errors, but they typically don't affect extraction

    let post_data = extract_post_data(&dom.document);

    // Generate output filename and extract date prefix
    let input_filename = file_path
        .file_stem()
        .unwrap_or_else(|| panic!("Failed to extract filename stem from: {}", file_name))
        .to_str()
        .unwrap_or_else(|| panic!("Filename contains invalid UTF-8: {}", file_name));
    let formatted_name = format_filename_date(input_filename);
    let output_filename = format!("{}.md", formatted_name);

    // Extract date prefix (YYYY-MM-DD) from formatted filename
    let date_prefix = if formatted_name.len() >= 10 && formatted_name.chars().nth(4) == Some('-') {
        &formatted_name[..10]
    } else {
        ""
    };

    let markdown_content = generate_markdown(&post_data, date_prefix);
    let output_path = Path::new(dest_dir).join(output_filename);

    // Write markdown file
    match std::fs::write(&output_path, markdown_content) {
        Err(why) => panic!("couldn't write {}: {}", output_path.display(), why),
        Ok(_) => println!("\tgenerated {:?}", output_path),
    }
}
