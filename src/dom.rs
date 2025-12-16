//! DOM manipulation and traversal helpers

use markup5ever_rcdom as rcdom;
use rcdom::{Handle, NodeData};

pub fn has_class(attrs: &[markup5ever::interface::Attribute], class_name: &str) -> bool {
    attrs
        .iter()
        .any(|attr| attr.name.local.as_ref() == "class" && attr.value.as_ref().contains(class_name))
}

pub fn has_attr(
    attrs: &[markup5ever::interface::Attribute],
    attr_name: &str,
    attr_value: &str,
) -> bool {
    attrs
        .iter()
        .any(|attr| attr.name.local.as_ref() == attr_name && attr.value.as_ref() == attr_value)
}

pub fn get_attr_value(
    attrs: &[markup5ever::interface::Attribute],
    attr_name: &str,
) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.name.local.as_ref() == attr_name)
        .map(|attr| attr.value.as_ref().to_string())
}

pub fn get_text_content(handle: &Handle) -> String {
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

pub fn get_text_content_formatted(handle: &Handle) -> String {
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
                        format_markdown_link(text, &href, &link_text);
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

pub fn find_parent_href(handle: &Handle) -> Option<String> {
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

/// Format a link as Markdown, adding spacing if needed
/// - If link text equals URL, uses angle bracket syntax: <URL>
/// - Otherwise uses full Markdown syntax: [text](URL)
/// - Adds space before link if text buffer doesn't end with whitespace
pub fn format_markdown_link(text: &mut String, href: &str, link_text: &str) {
    // Add space before link if needed
    if !text.is_empty() && !text.ends_with(|c: char| c.is_whitespace()) {
        text.push(' ');
    }
    // If link text is the same as URL, use angle bracket syntax
    // Otherwise use full Markdown link syntax
    if link_text == href {
        text.push('<');
        text.push_str(href);
        text.push('>');
    } else if !link_text.is_empty() {
        text.push('[');
        text.push_str(link_text);
        text.push_str("](");
        text.push_str(href);
        text.push(')');
    } else {
        // No link text, just use the URL in angle brackets
        text.push('<');
        text.push_str(href);
        text.push('>');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for format_markdown_link()
    #[test]
    fn test_format_markdown_link_url_equals_text() {
        let mut text = String::from("Check this out:");
        format_markdown_link(&mut text, "http://example.com", "http://example.com");
        assert_eq!(text, "Check this out: <http://example.com>");
    }

    #[test]
    fn test_format_markdown_link_different_text() {
        let mut text = String::from("Visit");
        format_markdown_link(&mut text, "http://example.com", "Example Site");
        assert_eq!(text, "Visit [Example Site](http://example.com)");
    }

    #[test]
    fn test_format_markdown_link_empty_text() {
        let mut text = String::from("Link:");
        format_markdown_link(&mut text, "http://example.com", "");
        assert_eq!(text, "Link: <http://example.com>");
    }

    #[test]
    fn test_format_markdown_link_empty_buffer() {
        let mut text = String::new();
        format_markdown_link(&mut text, "http://example.com", "Example");
        assert_eq!(text, "[Example](http://example.com)");
    }

    #[test]
    fn test_format_markdown_link_adds_space() {
        let mut text = String::from("Props.");
        format_markdown_link(&mut text, "http://example.com", "Example");
        assert_eq!(text, "Props. [Example](http://example.com)");
    }

    #[test]
    fn test_format_markdown_link_no_double_space() {
        let mut text = String::from("Check this ");
        format_markdown_link(&mut text, "http://example.com", "link");
        assert_eq!(text, "Check this [link](http://example.com)");
    }

    #[test]
    fn test_format_markdown_link_after_newline() {
        let mut text = String::from("Line one\n");
        format_markdown_link(&mut text, "http://example.com", "link");
        assert_eq!(text, "Line one\n[link](http://example.com)");
    }

    #[test]
    fn test_format_markdown_link_https() {
        let mut text = String::from("Secure:");
        format_markdown_link(&mut text, "https://example.com", "https://example.com");
        assert_eq!(text, "Secure: <https://example.com>");
    }
}
