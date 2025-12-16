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
use rcdom::{Handle, NodeData, RcDom};

use chrono::{DateTime, Utc};
use glob::glob;
use html_escape::decode_html_entities;

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
    let markdown_content = generate_markdown(&post_data);

    // Generate output filename
    let input_filename = file_path
        .file_stem()
        .unwrap_or_else(|| panic!("Failed to extract filename stem from: {}", file_name))
        .to_str()
        .unwrap_or_else(|| panic!("Filename contains invalid UTF-8: {}", file_name));
    let output_filename = format!("{}.md", format_filename_date(input_filename));
    let output_path = Path::new(dest_dir).join(output_filename);

    // Write markdown file
    match std::fs::write(&output_path, markdown_content) {
        Err(why) => panic!("couldn't write {}: {}", output_path.display(), why),
        Ok(_) => println!("\tgenerated {:?}", output_path),
    }
}

#[derive(Debug, Default)]
struct PostData {
    author: String,
    date: String,
    canonical_url: String,
    title: String,
    content: String,
    reshare_author: Option<String>,
    reshare_content: Option<String>,
    location: Option<String>,
    images: Vec<String>,
    video_url: Option<String>,
    links: Vec<(String, String)>, // (url, title)
    visibility: String,
    plus_ones: Vec<String>,
    comments: Vec<Comment>,
}

#[derive(Debug)]
struct Comment {
    author: String,
    date: String,
    content: String,
}

/// Extract structured data from the HTML document
fn extract_post_data(handle: &Handle) -> PostData {
    let mut post_data = PostData::default();

    // Find the main content div and extract data
    find_post_elements(handle, &mut post_data);

    post_data
}

/// Recursively search for post elements
fn find_post_elements(handle: &Handle, post_data: &mut PostData) {
    let node = handle;

    if let NodeData::Element {
        ref name,
        ref attrs,
        ..
    } = node.data
    {
        let attrs = attrs.borrow();
        let tag_name = name.local.as_ref();

        // Extract author from header
        if tag_name == "a" && has_class(&attrs, "author") && post_data.author.is_empty() {
            post_data.author = get_text_content(handle);
        }

        // Extract date/time and canonical URL from post header (not comments)
        // Post dates are in <a> tags that link to /posts/
        if tag_name == "a" && post_data.date.is_empty() {
            if let Some(href) = get_attr_value(&attrs, "href") {
                if href.contains("/posts/") {
                    let date_text = get_text_content(handle);
                    if !date_text.is_empty() {
                        post_data.date = convert_to_utc(&date_text);
                        post_data.canonical_url = href.clone();
                    }
                }
            }
        }

        // Extract main content
        if has_class(&attrs, "main-content") {
            post_data.content = get_text_content_formatted(handle);
        }

        // Extract title from HTML title tag
        if tag_name == "title" {
            let title_text = get_text_content(handle);
            if !title_text.is_empty() && title_text != "Google+ post" {
                post_data.title = title_text;
            }
        }

        // Extract location information
        if has_class(&attrs, "location") {
            let location_text = get_text_content(handle);
            post_data.location = Some(clean_location(&location_text));
        }

        // Extract images from albums or media links
        if tag_name == "img" && has_class(&attrs, "media") {
            if let Some(src) = get_attr_value(&attrs, "src") {
                post_data.images.push(src);
            }
        }

        // Extract video links
        if has_class(&attrs, "video-placeholder") {
            if let Some(href) = find_parent_href(handle) {
                post_data.video_url = Some(href);
            }
        }

        // Extract embedded links
        if tag_name == "a"
            && (has_attr(&attrs, "rel", "nofollow") || has_class(&attrs, "link-embed"))
        {
            if let Some(href) = get_attr_value(&attrs, "href") {
                let title = get_text_content(handle);
                post_data.links.push((href, title));
            }
        }

        // Extract visibility
        if has_class(&attrs, "visibility") {
            post_data.visibility = get_text_content(handle).replace("Shared with: ", "");
        }

        // Extract +1 information
        if has_class(&attrs, "plus-oners") {
            let plus_ones_text = get_text_content(handle);
            if plus_ones_text.starts_with("+1'd by: ") {
                let names = plus_ones_text.replace("+1'd by: ", "");
                post_data.plus_ones = names.split(", ").map(|s| s.to_string()).collect();
            }
        }

        // Extract reshare information
        if tag_name == "a" && has_class(&attrs, "reshare-attribution") {
            let attribution_text = get_text_content(handle);
            // Extract author name from "Originally shared by Author Name"
            post_data.reshare_author = Some(attribution_text.replace("Originally shared by ", ""));

            // Get reshare content from parent div's text nodes
            if let Some(reshare_content) = extract_reshare_content(handle) {
                post_data.reshare_content = Some(reshare_content);
            }
        }

        // Extract comments
        if has_class(&attrs, "comment") {
            if let Some(comment) = extract_comment(handle) {
                post_data.comments.push(comment);
            }
        }
    }

    // Recurse through children
    for child in node.children.borrow().iter() {
        find_post_elements(child, post_data);
    }
}

/// Extract comment data from a comment node
fn extract_comment(handle: &Handle) -> Option<Comment> {
    let mut author = String::new();
    let mut date = String::new();
    let mut content = String::new();

    fn extract_comment_parts(
        node: &Handle,
        author: &mut String,
        date: &mut String,
        content: &mut String,
    ) {
        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = &node.data
        {
            let attrs = attrs.borrow();
            let tag_name = name.local.as_ref();

            if tag_name == "a" && has_class(&attrs, "author") && author.is_empty() {
                *author = get_text_content(node);
            } else if has_class(&attrs, "time") && date.is_empty() {
                let date_text = get_text_content(node);
                // Comment dates have "- " prefix in the HTML, strip it
                let date_text = date_text.trim_start_matches("- ").trim();
                *date = convert_to_utc(date_text);
            } else if has_class(&attrs, "comment-content") && content.is_empty() {
                *content = get_text_content_formatted(node);
            }
        }

        for child in node.children.borrow().iter() {
            extract_comment_parts(child, author, date, content);
        }
    }

    extract_comment_parts(handle, &mut author, &mut date, &mut content);

    if !author.is_empty() && !content.is_empty() {
        Some(Comment {
            author,
            date,
            content,
        })
    } else {
        None
    }
}

/// Extract reshare content from the parent div of a reshare-attribution element
fn extract_reshare_content(reshare_attr_handle: &Handle) -> Option<String> {
    // Get parent element
    let parent_weak_opt = reshare_attr_handle.parent.take();
    let result = if let Some(ref weak) = parent_weak_opt {
        if let Some(parent_strong) = weak.upgrade() {
            // Extract text content from parent, excluding certain elements
            let content = extract_reshare_text(&parent_strong);
            let cleaned = content.trim().to_string();
            if cleaned.is_empty() {
                None
            } else {
                Some(cleaned)
            }
        } else {
            None
        }
    } else {
        None
    };
    // Restore parent reference
    reshare_attr_handle.parent.set(parent_weak_opt);
    result
}

/// Extract reshare text, excluding reshare-attribution and link-embed elements
fn extract_reshare_text(handle: &Handle) -> String {
    let mut text = String::new();

    fn collect_reshare_text(node: &Handle, text: &mut String) {
        match &node.data {
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let attrs = attrs.borrow();
                let tag_name = name.local.as_ref();

                // Skip reshare-attribution and link-embed elements
                if has_class(&attrs, "reshare-attribution") || has_class(&attrs, "link-embed") {
                    return;
                }

                // Handle br tags as newlines
                if tag_name == "br" {
                    text.push('\n');
                } else if tag_name == "a" {
                    // Handle links within reshared content - convert to Markdown
                    if let Some(href) = get_attr_value(&attrs, "href") {
                        let link_text = get_text_content(node);
                        // Add space before link if needed
                        if !text.is_empty() && !text.ends_with(|c: char| c.is_whitespace()) {
                            text.push(' ');
                        }
                        // If link text is the same as URL, use angle bracket syntax
                        // Otherwise use full Markdown link syntax
                        if link_text == href {
                            text.push('<');
                            text.push_str(&href);
                            text.push('>');
                        } else if !link_text.is_empty() {
                            text.push('[');
                            text.push_str(&link_text);
                            text.push_str("](");
                            text.push_str(&href);
                            text.push(')');
                        } else {
                            // No link text, just use the URL in angle brackets
                            text.push('<');
                            text.push_str(&href);
                            text.push('>');
                        }
                    } else {
                        // No href attribute, just extract text
                        for child in node.children.borrow().iter() {
                            collect_reshare_text(child, text);
                        }
                    }
                } else {
                    // Recurse into other elements
                    for child in node.children.borrow().iter() {
                        collect_reshare_text(child, text);
                    }
                }
            }
            NodeData::Text { ref contents } => {
                text.push_str(&contents.borrow());
            }
            _ => {
                for child in node.children.borrow().iter() {
                    collect_reshare_text(child, text);
                }
            }
        }
    }

    collect_reshare_text(handle, &mut text);
    text.trim().to_string()
}

/// Generate markdown from post data
fn generate_markdown(post_data: &PostData) -> String {
    let mut markdown = String::new();

    // Generate TOML front matter
    markdown.push_str("+++\n");

    // Title - use post title if available, otherwise use truncated content
    let title = if !post_data.title.is_empty() {
        let cleaned = clean_title(&post_data.title);
        escape_toml_string(&cleaned)
    } else if !post_data.content.is_empty() {
        let truncated = post_data.content.chars().take(50).collect::<String>();
        escape_toml_string(&format!("{}...", truncated.trim()))
    } else {
        String::from("Google+ Post")
    };
    markdown.push_str(&format!("title = \"{}\"\n", title));

    // Date - use raw format from Google+ for now
    if !post_data.date.is_empty() {
        markdown.push_str(&format!(
            "date = \"{}\"\n",
            escape_toml_string(&post_data.date)
        ));
    } else {
        markdown.push_str("date = \"\"\n");
    }

    markdown.push_str("draft = false\n");

    // Description - first 150 chars of content
    let description = if !post_data.content.is_empty() {
        let truncated = post_data.content.chars().take(150).collect::<String>();
        escape_toml_string(truncated.trim())
    } else {
        String::from("")
    };
    markdown.push_str(&format!("# description = \"{}\"\n", description));

    // Canonical URL - original Google+ post URL
    if !post_data.canonical_url.is_empty() {
        markdown.push_str(&format!(
            "canonicalURL = \"{}\"\n",
            escape_toml_string(&post_data.canonical_url)
        ));
        markdown.push_str("ShowCanonicalLink = true\n");
    } else {
        markdown.push_str("canonicalURL = \"\"\n");
        markdown.push_str("ShowCanonicalLink = false\n");
    }

    // Cover image settings
    markdown.push_str("# cover.image = \"/posts/\"\n");
    markdown.push_str("cover.hidden = true\n");

    // Optional metadata as comments
    if !post_data.author.is_empty() {
        markdown.push_str(&format!(
            "# author = \"{}\"\n",
            escape_toml_string(&post_data.author)
        ));
    }
    markdown.push_str("# keywords = [\"google-plus\", \"archive\"]\n");
    markdown.push_str("tags = [\"google-plus\"]\n");

    markdown.push_str("# ShowToc = false\n");
    markdown.push_str("+++\n\n");

    // Post metadata section
    let mut metadata_parts = Vec::new();
    if let Some(location) = &post_data.location {
        metadata_parts.push(format!("**Location:** {}", location));
    }
    if !metadata_parts.is_empty() {
        markdown.push_str(&metadata_parts.join(" | "));
        markdown.push_str("\n\n---\n\n");
    }

    // Add main content
    if !post_data.content.is_empty() {
        markdown.push_str(&post_data.content);
        markdown.push_str("\n\n");
    }

    // Add reshare information
    if let Some(reshare_author) = &post_data.reshare_author {
        markdown.push_str(&format!("**Originally shared by {}**\n\n", reshare_author));
        if let Some(reshare_content) = &post_data.reshare_content {
            markdown.push_str(reshare_content);
            markdown.push_str("\n\n");
        }
    }

    // Add images
    if !post_data.images.is_empty() {
        markdown.push_str("## Images\n\n");
        for image_url in &post_data.images {
            markdown.push_str(&format!("![Image]({})\n\n", image_url));
        }
    }

    // Add video
    if let Some(video_url) = &post_data.video_url {
        markdown.push_str(&format!("## Video\n\n[Watch Video]({})\n\n", video_url));
    }

    // Add links
    if !post_data.links.is_empty() {
        markdown.push_str("## Links\n\n");
        for (url, title) in &post_data.links {
            let link_text = if title.is_empty() { url } else { title };
            markdown.push_str(&format!("- [{}]({})\n", link_text, url));
        }
        markdown.push('\n');
    }

    // Add shared with
    if !post_data.visibility.is_empty() {
        markdown.push_str(&format!("**Shared with:** {}\n\n", post_data.visibility));
    }

    // Add +1s
    if !post_data.plus_ones.is_empty() {
        markdown.push_str(&format!(
            "**+1'd by:** {}\n\n",
            post_data.plus_ones.join(", ")
        ));
    }

    // Add comments
    if !post_data.comments.is_empty() {
        markdown.push_str("## Comments\n\n");
        for comment in &post_data.comments {
            markdown.push_str(&format!("**{}**", comment.author));
            if !comment.date.is_empty() {
                markdown.push_str(&format!(" - {}", comment.date));
            }
            markdown.push_str(&format!("\n\n{}\n\n---\n\n", comment.content));
        }
    }

    format!("{}\n", markdown.trim_end())
}

// Helper functions

/// Escape double quotes and backslashes for TOML basic string values
/// Also replaces newlines with spaces to ensure single-line TOML strings
fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

/// Clean up title text by decoding HTML entities and stripping HTML tags
/// This handles double-encoded entities from Google+ Takeout HTML
fn clean_title(title: &str) -> String {
    // Decode HTML entities (this handles &#39;, &quot;, &amp;, etc.)
    let decoded = decode_html_entities(title).to_string();

    // Strip HTML tags using a simple regex-like approach
    // This handles cases like <br>, <br/>, <b>, etc.
    let mut result = String::new();
    let mut in_tag = false;

    for c in decoded.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                // Add a space where tags were to avoid word concatenation
                result.push(' ');
            }
            _ if !in_tag => result.push(c),
            _ => {} // Skip characters inside tags
        }
    }

    // Clean up multiple spaces and trim
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Clean up location text to ensure proper spacing before "Address"
/// Google+ sometimes concatenates location data without proper spacing
fn clean_location(location: &str) -> String {
    // Ensure there's a space before "Address" if it's not already there
    let mut result = location.to_string();

    // Check for "Address" without a preceding space
    if let Some(pos) = result.find("Address") {
        if pos > 0 {
            let before = &result[..pos];
            if !before.ends_with(|c: char| c.is_whitespace()) {
                result.insert(pos, ' ');
            }
        }
    }

    result
}

/// Convert Google+ datetime string to UTC
/// Input format: "YYYY-MM-DD HH:MM:SS±HHMM" (e.g., "2011-08-14 20:39:28-0700")
/// Output format: ISO 8601 UTC (e.g., "2011-08-15T03:39:28Z")
fn convert_to_utc(datetime_str: &str) -> String {
    // Parse the datetime with timezone offset
    match DateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S%z") {
        Ok(dt) => {
            // Convert to UTC
            let utc_dt: DateTime<Utc> = dt.with_timezone(&Utc);
            // Format as ISO 8601 with Z suffix
            utc_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
        }
        Err(_) => {
            // If parsing fails, return original string
            datetime_str.to_string()
        }
    }
}

/// Format filename date from YYYYMMDD to YYYY-MM-DD and clean up spacing
/// Input: "20110814 - Today is my first day..." or any other filename
/// Output: "2011-08-14-Today_is_my_first_day..."
/// - Converts YYYYMMDD to YYYY-MM-DD
/// - Replaces " - " with "-"
/// - Replaces remaining spaces with underscores
/// - Removes @, !, and # symbols
fn format_filename_date(filename: &str) -> String {
    // Check if filename starts with 8 digits
    if filename.len() >= 8 && filename.chars().take(8).all(|c| c.is_ascii_digit()) {
        let year = &filename[0..4];
        let month = &filename[4..6];
        let day = &filename[6..8];
        let rest = &filename[8..];

        // Replace " - " with "-", replace spaces with underscores, and remove @, !, #, &, (, )
        let rest_formatted = rest
            .trim_start_matches(" - ")
            .replace(' ', "_")
            .replace(['@', '!', '#', '&', '(', ')'], "");

        format!("{}-{}-{}-{}", year, month, day, rest_formatted)
    } else {
        // For non-date filenames, replace spaces with underscores and remove @, !, #, &, (, )
        filename
            .replace(' ', "_")
            .replace(['@', '!', '#', '&', '(', ')'], "")
    }
}

fn has_class(attrs: &[markup5ever::interface::Attribute], class_name: &str) -> bool {
    attrs
        .iter()
        .any(|attr| attr.name.local.as_ref() == "class" && attr.value.as_ref().contains(class_name))
}

fn has_attr(
    attrs: &[markup5ever::interface::Attribute],
    attr_name: &str,
    attr_value: &str,
) -> bool {
    attrs
        .iter()
        .any(|attr| attr.name.local.as_ref() == attr_name && attr.value.as_ref() == attr_value)
}

fn get_attr_value(attrs: &[markup5ever::interface::Attribute], attr_name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.name.local.as_ref() == attr_name)
        .map(|attr| attr.value.as_ref().to_string())
}

fn get_text_content(handle: &Handle) -> String {
    let mut text = String::new();

    fn collect_text(node: &Handle, text: &mut String) {
        match &node.data {
            NodeData::Text { ref contents } => {
                text.push_str(&contents.borrow());
            }
            _ => {
                for child in node.children.borrow().iter() {
                    collect_text(child, text);
                }
            }
        }
    }

    collect_text(handle, &mut text);
    text.trim().to_string()
}

fn get_text_content_formatted(handle: &Handle) -> String {
    let mut text = String::new();

    fn collect_text_formatted(node: &Handle, text: &mut String) {
        match &node.data {
            NodeData::Text { ref contents } => {
                text.push_str(&contents.borrow());
            }
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let tag_name = name.local.as_ref();
                if tag_name == "br" {
                    text.push('\n');
                } else if tag_name == "a" {
                    // Handle links within content - convert to Markdown
                    let attrs = attrs.borrow();
                    if let Some(href) = get_attr_value(&attrs, "href") {
                        let link_text = get_text_content(node);
                        // Add space before link if needed
                        if !text.is_empty() && !text.ends_with(|c: char| c.is_whitespace()) {
                            text.push(' ');
                        }
                        // If link text is the same as URL, use angle bracket syntax
                        // Otherwise use full Markdown link syntax
                        if link_text == href {
                            text.push('<');
                            text.push_str(&href);
                            text.push('>');
                        } else if !link_text.is_empty() {
                            text.push('[');
                            text.push_str(&link_text);
                            text.push_str("](");
                            text.push_str(&href);
                            text.push(')');
                        } else {
                            // No link text, just use the URL in angle brackets
                            text.push('<');
                            text.push_str(&href);
                            text.push('>');
                        }
                    } else {
                        // No href attribute, just extract text
                        for child in node.children.borrow().iter() {
                            collect_text_formatted(child, text);
                        }
                    }
                } else {
                    for child in node.children.borrow().iter() {
                        collect_text_formatted(child, text);
                    }
                }
            }
            _ => {
                for child in node.children.borrow().iter() {
                    collect_text_formatted(child, text);
                }
            }
        }
    }

    collect_text_formatted(handle, &mut text);
    text.trim().to_string()
}

fn find_parent_href(handle: &Handle) -> Option<String> {
    // Look for href in parent elements
    fn search_parents(node: &Handle) -> Option<String> {
        // Temporarily take parent ref, use it, then restore it
        let parent_weak_opt = node.parent.take();
        let result = if let Some(ref weak) = parent_weak_opt {
            if let Some(parent_strong) = weak.upgrade() {
                match &parent_strong.data {
                    NodeData::Element { ref attrs, .. } => {
                        if let Some(href) = get_attr_value(&attrs.borrow(), "href") {
                            Some(href)
                        } else {
                            search_parents(&parent_strong)
                        }
                    }
                    _ => search_parents(&parent_strong),
                }
            } else {
                None
            }
        } else {
            None
        };
        // Restore parent reference
        node.parent.set(parent_weak_opt);
        result
    }

    search_parents(handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for escape_toml_string()
    #[test]
    fn test_escape_toml_string_simple() {
        assert_eq!(escape_toml_string("hello world"), "hello world");
    }

    #[test]
    fn test_escape_toml_string_empty() {
        assert_eq!(escape_toml_string(""), "");
    }

    #[test]
    fn test_escape_toml_string_quotes() {
        assert_eq!(
            escape_toml_string("He said \"hello\""),
            "He said \\\"hello\\\""
        );
    }

    #[test]
    fn test_escape_toml_string_backslashes() {
        assert_eq!(escape_toml_string("C:\\Users\\path"), "C:\\\\Users\\\\path");
    }

    #[test]
    fn test_escape_toml_string_mixed() {
        assert_eq!(
            escape_toml_string("\"quote\\\" and \\backslash"),
            "\\\"quote\\\\\\\" and \\\\backslash"
        );
    }

    #[test]
    fn test_escape_toml_string_unicode() {
        assert_eq!(escape_toml_string("Hello 👋 世界"), "Hello 👋 世界");
    }

    #[test]
    fn test_escape_toml_string_only_quotes() {
        assert_eq!(escape_toml_string("\"\"\""), "\\\"\\\"\\\"");
    }

    #[test]
    fn test_escape_toml_string_only_backslashes() {
        assert_eq!(escape_toml_string("\\\\\\"), "\\\\\\\\\\\\");
    }

    #[test]
    fn test_escape_toml_string_newlines() {
        assert_eq!(
            escape_toml_string("Line one\nLine two\nLine three"),
            "Line one Line two Line three"
        );
    }

    #[test]
    fn test_escape_toml_string_mixed_newlines() {
        assert_eq!(
            escape_toml_string("He said \"hello\"\nAnd then left"),
            "He said \\\"hello\\\" And then left"
        );
    }

    #[test]
    fn test_escape_toml_string_carriage_return() {
        assert_eq!(
            escape_toml_string("Windows\r\nStyle\r\nNewlines"),
            "Windows  Style  Newlines"
        );
    }

    // Tests for convert_to_utc()
    #[test]
    fn test_convert_to_utc_negative_offset() {
        assert_eq!(
            convert_to_utc("2011-08-14 20:39:28-0700"),
            "2011-08-15T03:39:28Z"
        );
    }

    #[test]
    fn test_convert_to_utc_positive_offset() {
        assert_eq!(
            convert_to_utc("2024-01-15 14:30:00+0530"),
            "2024-01-15T09:00:00Z"
        );
    }

    #[test]
    fn test_convert_to_utc_zero_offset() {
        assert_eq!(
            convert_to_utc("2024-06-15 12:00:00+0000"),
            "2024-06-15T12:00:00Z"
        );
    }

    #[test]
    fn test_convert_to_utc_midnight_boundary() {
        // 11:30 PM PST becomes 7:30 AM UTC next day
        assert_eq!(
            convert_to_utc("2024-01-15 23:30:00-0800"),
            "2024-01-16T07:30:00Z"
        );
    }

    #[test]
    fn test_convert_to_utc_date_boundary_backward() {
        // 2 AM IST becomes previous day in UTC
        assert_eq!(
            convert_to_utc("2024-01-16 02:00:00+0530"),
            "2024-01-15T20:30:00Z"
        );
    }

    #[test]
    fn test_convert_to_utc_invalid_format() {
        // Should return original string on parse error
        assert_eq!(convert_to_utc("not a date"), "not a date");
    }

    #[test]
    fn test_convert_to_utc_empty_string() {
        assert_eq!(convert_to_utc(""), "");
    }

    #[test]
    fn test_convert_to_utc_wrong_format() {
        // ISO format instead of expected format
        assert_eq!(
            convert_to_utc("2024-01-15T14:30:00Z"),
            "2024-01-15T14:30:00Z"
        );
    }

    // Tests for format_filename_date()
    #[test]
    fn test_format_filename_date_standard() {
        assert_eq!(
            format_filename_date("20110814 - Today is my first day"),
            "2011-08-14-Today_is_my_first_day"
        );
    }

    #[test]
    fn test_format_filename_date_no_separator() {
        // Function always adds dash after date
        assert_eq!(format_filename_date("20110814Today"), "2011-08-14-Today");
    }

    #[test]
    fn test_format_filename_date_multiple_spaces() {
        assert_eq!(
            format_filename_date("20110814 - Multiple Word Title Here"),
            "2011-08-14-Multiple_Word_Title_Here"
        );
    }

    #[test]
    fn test_format_filename_date_just_date() {
        assert_eq!(format_filename_date("20110814"), "2011-08-14-");
    }

    #[test]
    fn test_format_filename_date_non_date() {
        assert_eq!(format_filename_date("random file name"), "random_file_name");
    }

    #[test]
    fn test_format_filename_date_short_filename() {
        assert_eq!(format_filename_date("short"), "short");
    }

    #[test]
    fn test_format_filename_date_partial_date() {
        // 7 digits, not 8 - should not be treated as date
        assert_eq!(format_filename_date("2011081 - test"), "2011081_-_test");
    }

    #[test]
    fn test_format_filename_date_with_extension() {
        // This tests just the stem, but good to verify
        assert_eq!(
            format_filename_date("20110814 - Post Title"),
            "2011-08-14-Post_Title"
        );
    }

    #[test]
    fn test_format_filename_date_no_dash_separator() {
        // Has date but no " - " separator (just space), so space becomes underscore
        assert_eq!(
            format_filename_date("20110814 Post Title"),
            "2011-08-14-_Post_Title"
        );
    }

    #[test]
    fn test_format_filename_date_empty() {
        assert_eq!(format_filename_date(""), "");
    }

    #[test]
    fn test_format_filename_date_special_chars() {
        assert_eq!(
            format_filename_date("20110814 - Post with (parentheses) & stuff"),
            "2011-08-14-Post_with_parentheses__stuff"
        );
    }

    #[test]
    fn test_format_filename_date_removes_at_symbol() {
        assert_eq!(
            format_filename_date("20110814 - Email @someone about this"),
            "2011-08-14-Email_someone_about_this"
        );
    }

    #[test]
    fn test_format_filename_date_removes_exclamation() {
        assert_eq!(
            format_filename_date("20110814 - Wow! This is cool!"),
            "2011-08-14-Wow_This_is_cool"
        );
    }

    #[test]
    fn test_format_filename_date_removes_hash() {
        assert_eq!(
            format_filename_date("20110814 - Post about #hashtags and #coding"),
            "2011-08-14-Post_about_hashtags_and_coding"
        );
    }

    #[test]
    fn test_format_filename_date_removes_all_symbols() {
        assert_eq!(
            format_filename_date("20110814 - Wow! Email @user about #topic"),
            "2011-08-14-Wow_Email_user_about_topic"
        );
    }

    #[test]
    fn test_format_filename_date_removes_symbols_non_date() {
        assert_eq!(
            format_filename_date("My post @home about #things!"),
            "My_post_home_about_things"
        );
    }

    // Tests for clean_title()
    #[test]
    fn test_clean_title_simple() {
        assert_eq!(clean_title("Hello World"), "Hello World");
    }

    #[test]
    fn test_clean_title_empty() {
        assert_eq!(clean_title(""), "");
    }

    #[test]
    fn test_clean_title_apostrophe_entity() {
        assert_eq!(clean_title("What&#39;d you say"), "What'd you say");
    }

    #[test]
    fn test_clean_title_quote_entity() {
        assert_eq!(clean_title("&quot;Hello World&quot;"), "\"Hello World\"");
    }

    #[test]
    fn test_clean_title_ampersand_entity() {
        assert_eq!(clean_title("Penn &amp; Teller"), "Penn & Teller");
    }

    #[test]
    fn test_clean_title_less_than_greater_than() {
        // When &lt;tag&gt; is decoded, it becomes <tag> which is then stripped
        // This is correct behavior - encoded HTML tags should be removed
        assert_eq!(clean_title("&lt;tag&gt;"), "");
    }

    #[test]
    fn test_clean_title_encoded_text_with_angle_brackets() {
        // To preserve <tag> as text, it needs to be double-encoded
        // But in practice, Google+ doesn't do this, so we strip tags
        assert_eq!(clean_title("Code example: &lt;div&gt;"), "Code example:");
    }

    #[test]
    fn test_clean_title_br_tag() {
        assert_eq!(clean_title("Line one<br>Line two"), "Line one Line two");
    }

    #[test]
    fn test_clean_title_br_self_closing() {
        assert_eq!(clean_title("Line one<br/>Line two"), "Line one Line two");
    }

    #[test]
    fn test_clean_title_multiple_br_tags() {
        assert_eq!(clean_title("Line one<br><br>Line two"), "Line one Line two");
    }

    #[test]
    fn test_clean_title_bold_tag() {
        assert_eq!(clean_title("This is <b>bold</b> text"), "This is bold text");
    }

    #[test]
    fn test_clean_title_mixed_entities_and_tags() {
        assert_eq!(
            clean_title("What&#39;d you say again?<br><br>This is fabulous"),
            "What'd you say again? This is fabulous"
        );
    }

    #[test]
    fn test_clean_title_real_world_example_1() {
        // From the actual Google+ export
        assert_eq!(
            clean_title("Scott&#39;s brother Greg and his girl friend"),
            "Scott's brother Greg and his girl friend"
        );
    }

    #[test]
    fn test_clean_title_real_world_example_2() {
        // From the actual Google+ export
        assert_eq!(
            clean_title("Penn &amp; Teller rock!"),
            "Penn & Teller rock!"
        );
    }

    #[test]
    fn test_clean_title_real_world_example_3() {
        // From the actual Google+ export
        assert_eq!(
            clean_title("&quot;Lessons Learned Developing Software for Space Vehicles&quot;"),
            "\"Lessons Learned Developing Software for Space Vehicles\""
        );
    }

    #[test]
    fn test_clean_title_multiple_spaces() {
        // After tag removal, multiple spaces should be normalized
        assert_eq!(clean_title("Hello<br>  <br>  World"), "Hello World");
    }

    #[test]
    fn test_clean_title_nested_tags() {
        assert_eq!(
            clean_title("This is <b><i>nested</i></b> text"),
            "This is nested text"
        );
    }

    #[test]
    fn test_clean_title_tag_with_attributes() {
        assert_eq!(
            clean_title("Click <a href=\"http://example.com\">here</a>"),
            "Click here"
        );
    }

    #[test]
    fn test_clean_title_unicode() {
        assert_eq!(clean_title("Hello 👋 世界"), "Hello 👋 世界");
    }

    #[test]
    fn test_clean_title_unicode_with_entities() {
        assert_eq!(
            clean_title("Hello 👋 世界&#39;s best"),
            "Hello 👋 世界's best"
        );
    }

    #[test]
    fn test_clean_title_only_tags() {
        assert_eq!(clean_title("<br><br><br>"), "");
    }

    #[test]
    fn test_clean_title_truncated_entity() {
        // Handle case where HTML title was truncated mid-entity
        // The library should handle this gracefully
        assert_eq!(clean_title("Test&#3"), "Test&#3");
    }

    // Tests for clean_location()
    #[test]
    fn test_clean_location_with_missing_space() {
        assert_eq!(
            clean_location("123 Main StreetAddress: City, State"),
            "123 Main Street Address: City, State"
        );
    }

    #[test]
    fn test_clean_location_with_proper_space() {
        assert_eq!(
            clean_location("123 Main Street Address: City, State"),
            "123 Main Street Address: City, State"
        );
    }

    #[test]
    fn test_clean_location_no_address() {
        assert_eq!(clean_location("New York, NY"), "New York, NY");
    }

    #[test]
    fn test_clean_location_empty() {
        assert_eq!(clean_location(""), "");
    }

    #[test]
    fn test_clean_location_address_at_start() {
        assert_eq!(
            clean_location("Address: 123 Main Street"),
            "Address: 123 Main Street"
        );
    }

    #[test]
    fn test_clean_location_multiple_occurrences() {
        // Only fixes the first occurrence
        assert_eq!(
            clean_location("HomeAddress: 123, WorkAddress: 456"),
            "Home Address: 123, WorkAddress: 456"
        );
    }
}
