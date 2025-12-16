//! HTML parsing and data extraction

use markup5ever_rcdom as rcdom;
use rcdom::{Handle, NodeData};

use crate::dom::{
    find_parent_href, format_markdown_link, get_attr_value, get_text_content,
    get_text_content_formatted, has_attr, has_class,
};
use crate::models::{Comment, PostData};
use crate::utils::{clean_location, convert_to_utc};

/// Extract structured data from the HTML document
pub fn extract_post_data(handle: &Handle) -> PostData {
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
                        format_markdown_link(text, &href, &link_text);
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
