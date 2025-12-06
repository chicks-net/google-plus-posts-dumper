//! # google-plus-posts-dumper
//!
//! Parse HTMl from posts in google+ data dump
//! to generate hugu-friendly Markdown.
//!
//! Status: in develoment
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
use std::fs::{File, write};
use std::path::Path;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use rcdom::{Handle, NodeData, RcDom};
use markup5ever::interface::Attribute;

use glob::glob;

fn main() {
    // get directory argument and verify that it is actually a directory
    let args: Vec<String> = env::args().collect();
    // dbg!(args);
    let base_path_arg = &args[1];
    let base_path = Path::new(base_path_arg);
    assert_dir(base_path);

    // destination directory
    let dest_path_arg = &args[2];
    let dest_path = Path::new(dest_path_arg);
    assert_dir(dest_path);

    // find posts directory - either Google+ structure or direct examples
    let posts_path = Path::new(base_path).join("Google+ Stream/Posts");
    let posts_path_string = if posts_path.exists() && posts_path.is_dir() {
        // Original Google+ Takeout structure
        posts_path.to_str().unwrap().to_string()
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

    println!("processing {:?}",file_name);
    println!("\tinto {:?}",dest_dir);

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file_handle = match File::open(file_path) {
        Err(why) => panic!("couldn't open {}: {}", file_name, why),
        Ok(file_handle) => file_handle,
    };

    // HTML parsing
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut file_handle)
        .unwrap();
    // Parse the document to extract post data

    if !dom.errors.is_empty() {
        println!("\nParse errors that may not matter:");
        for err in dom.errors.iter() {
            println!("    {}", err);
        }
    }

    let post_data = extract_post_data(&dom.document);
    let markdown_content = generate_markdown(&post_data);

    // Generate output filename
    let input_filename = file_path.file_stem().unwrap().to_str().unwrap();
    let output_filename = format!("{}.md", input_filename);
    let output_path = Path::new(dest_dir).join(output_filename);

    // Write markdown file
    match write(&output_path, markdown_content) {
        Err(why) => panic!("couldn't write {}: {}", output_path.display(), why),
        Ok(_) => println!("\tgenerated {:?}", output_path),
    }
}

#[derive(Debug, Default)]
struct PostData {
    author: String,
    date: String,
    title: String,
    content: String,
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

    if let NodeData::Element { ref name, ref attrs, .. } = node.data {
        let attrs = attrs.borrow();
        let tag_name = name.local.as_ref();

        // Extract author from header
        if tag_name == "a" && has_class(&attrs, "author") {
            post_data.author = get_text_content(handle);
        }

        // Extract date/time
        if has_class(&attrs, "time") {
            let date_text = get_text_content(handle);
            if !date_text.is_empty() && post_data.date.is_empty() {
                post_data.date = date_text;
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
            post_data.location = Some(get_text_content(handle));
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
        if tag_name == "a" && has_attr(&attrs, "rel", "nofollow") {
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

    fn extract_comment_parts(node: &Handle, author: &mut String, date: &mut String, content: &mut String) {
        if let NodeData::Element { ref name, ref attrs, .. } = &node.data {
            let attrs = attrs.borrow();
            let tag_name = name.local.as_ref();

            if tag_name == "a" && has_class(&attrs, "author") && author.is_empty() {
                *author = get_text_content(node);
            } else if has_class(&attrs, "time") && date.is_empty() {
                *date = get_text_content(node);
            } else if has_class(&attrs, "comment-content") && content.is_empty() {
                *content = get_text_content(node);
            }
        }

        for child in node.children.borrow().iter() {
            extract_comment_parts(child, author, date, content);
        }
    }

    extract_comment_parts(handle, &mut author, &mut date, &mut content);

    if !author.is_empty() && !content.is_empty() {
        Some(Comment { author, date, content })
    } else {
        None
    }
}

/// Generate markdown from post data
fn generate_markdown(post_data: &PostData) -> String {
    let mut markdown = String::new();

    // Generate TOML front matter
    markdown.push_str("+++\n");

    // Title - use post title if available, otherwise use truncated content
    let title = if !post_data.title.is_empty() {
        escape_toml_string(&post_data.title)
    } else if !post_data.content.is_empty() {
        let truncated = post_data.content.chars().take(50).collect::<String>();
        escape_toml_string(&format!("{}...", truncated.trim()))
    } else {
        String::from("Google+ Post")
    };
    markdown.push_str(&format!("title = '{}'\n", title));

    // Date - use raw format from Google+ for now
    if !post_data.date.is_empty() {
        markdown.push_str(&format!("date = '{}'\n", escape_toml_string(&post_data.date)));
    } else {
        markdown.push_str("date = ''\n");
    }

    markdown.push_str("draft = false\n");

    // Description - first 150 chars of content
    let description = if !post_data.content.is_empty() {
        let truncated = post_data.content.chars().take(150).collect::<String>();
        escape_toml_string(truncated.trim())
    } else {
        String::from("")
    };
    markdown.push_str(&format!("description = '{}'\n", description));

    // Canonical URL - leave empty for now
    markdown.push_str("canonicalURL = ''\n");
    markdown.push_str("ShowCanonicalLink = false\n");

    // Cover image settings
    markdown.push_str("# cover.image = '/posts/'\n");
    markdown.push_str("cover.hidden = true\n");

    // Optional metadata as comments
    markdown.push_str("# keywords = [\"google-plus\", \"archive\"]\n");
    markdown.push_str("# tags = [\"google-plus\"");
    if !post_data.visibility.is_empty() {
        markdown.push_str(&format!(", \"{}\"", escape_toml_string(&post_data.visibility.to_lowercase())));
    }
    if post_data.location.is_some() {
        markdown.push_str(", \"location\"");
    }
    markdown.push_str("]\n");

    markdown.push_str("# ShowToc = false\n");
    markdown.push_str("+++\n\n");

    // Post metadata section
    let mut metadata_parts = Vec::new();
    if !post_data.author.is_empty() {
        metadata_parts.push(format!("**Author:** {}", post_data.author));
    }
    if let Some(location) = &post_data.location {
        metadata_parts.push(format!("**Location:** {}", location));
    }
    if !post_data.visibility.is_empty() {
        metadata_parts.push(format!("**Shared with:** {}", post_data.visibility));
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

    // Add +1s
    if !post_data.plus_ones.is_empty() {
        markdown.push_str(&format!("**+1'd by:** {}\n\n", post_data.plus_ones.join(", ")));
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

    markdown
}

/// Helper functions

/// Escape single quotes for TOML string values
fn escape_toml_string(s: &str) -> String {
    s.replace('\'', "''")
}

fn has_class(attrs: &[Attribute], class_name: &str) -> bool {
    attrs.iter().any(|attr| {
        attr.name.local.as_ref() == "class" && attr.value.as_ref().contains(class_name)
    })
}

fn has_attr(attrs: &[Attribute], attr_name: &str, attr_value: &str) -> bool {
    attrs.iter().any(|attr| {
        attr.name.local.as_ref() == attr_name && attr.value.as_ref() == attr_value
    })
}

fn get_attr_value(attrs: &[Attribute], attr_name: &str) -> Option<String> {
    attrs.iter()
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
            NodeData::Element { ref name, .. } => {
                let tag_name = name.local.as_ref();
                if tag_name == "br" {
                    text.push('\n');
                } else if tag_name == "a" {
                    // Handle links within content
                    for child in node.children.borrow().iter() {
                        collect_text_formatted(child, text);
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
        if let Some(parent) = node.parent.take() {
            if let Some(parent_strong) = parent.upgrade() {
                match &parent_strong.data {
                    NodeData::Element { ref attrs, .. } => {
                        if let Some(href) = get_attr_value(&attrs.borrow(), "href") {
                            return Some(href);
                        }
                        return search_parents(&parent_strong);
                    }
                    _ => return search_parents(&parent_strong),
                }
            }
        }
        None
    }

    search_parents(handle)
}
