//! Markdown generation from post data

use crate::models::PostData;
use crate::utils::{clean_title, escape_toml_string};

/// Generate markdown from post data
pub fn generate_markdown(post_data: &PostData) -> String {
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
