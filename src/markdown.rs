//! Markdown generation from post data

use crate::models::PostData;
use crate::utils::{clean_title, escape_toml_string};
use std::path::Path;

/// Transform an image path to /posts/YYYY-MM-DD-filename.ext format
///
/// # Arguments
/// * `image_path` - The original image path from the HTML
/// * `date_prefix` - The date prefix (YYYY-MM-DD) from the post filename
///
/// # Returns
/// The transformed path in the format /posts/YYYY-MM-DD-filename.ext
fn transform_image_path(image_path: &str, date_prefix: &str) -> String {
    // Extract just the filename from the path
    let filename = Path::new(image_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(image_path);

    format!("/posts/{}-{}", date_prefix, filename)
}

/// Generate markdown from post data
///
/// # Arguments
/// * `post_data` - The post data to generate markdown from
/// * `date_prefix` - The date prefix (YYYY-MM-DD) for image paths
pub fn generate_markdown(post_data: &PostData, date_prefix: &str) -> String {
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
            let transformed_path = transform_image_path(image_url, date_prefix);
            markdown.push_str(&format!("![Image]({})\n\n", transformed_path));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_image_path_standard() {
        assert_eq!(
            transform_image_path("../Photos/Photos from posts/pretty/image.jpg", "2011-11-04"),
            "/posts/2011-11-04-image.jpg"
        );
    }

    #[test]
    fn test_transform_image_path_complex_path() {
        assert_eq!(
            transform_image_path(
                "../Photos/Photos%20from%20posts/Vasquez%20Rocks/183zw3ui6c0yq.jpg",
                "2012-11-01"
            ),
            "/posts/2012-11-01-183zw3ui6c0yq.jpg"
        );
    }

    #[test]
    fn test_transform_image_path_simple_filename() {
        assert_eq!(
            transform_image_path("image.jpg", "2013-07-13"),
            "/posts/2013-07-13-image.jpg"
        );
    }

    #[test]
    fn test_transform_image_path_empty_date() {
        assert_eq!(transform_image_path("image.jpg", ""), "/posts/-image.jpg");
    }

    #[test]
    fn test_transform_image_path_url() {
        assert_eq!(
            transform_image_path("http://example.com/path/image.png", "2013-07-13"),
            "/posts/2013-07-13-image.png"
        );
    }

    #[test]
    fn test_transform_image_path_windows_path() {
        // On non-Windows systems, Path won't recognize Windows separators
        // and will treat the entire string as a filename
        #[cfg(windows)]
        {
            assert_eq!(
                transform_image_path("C:\\Photos\\image.jpg", "2015-01-01"),
                "/posts/2015-01-01-image.jpg"
            );
        }
        #[cfg(not(windows))]
        {
            // On Unix systems, backslashes are valid filename characters, not separators
            assert_eq!(
                transform_image_path("C:\\Photos\\image.jpg", "2015-01-01"),
                "/posts/2015-01-01-C:\\Photos\\image.jpg"
            );
        }
    }

    #[test]
    fn test_transform_image_path_absolute_path() {
        assert_eq!(
            transform_image_path("/var/data/photos/image.jpg", "2016-06-15"),
            "/posts/2016-06-15-image.jpg"
        );
    }

    #[test]
    fn test_transform_image_path_special_chars() {
        assert_eq!(
            transform_image_path("../Photos/my photo (1).jpg", "2017-03-20"),
            "/posts/2017-03-20-my photo (1).jpg"
        );
    }

    #[test]
    fn test_transform_image_path_no_extension() {
        assert_eq!(
            transform_image_path("../Photos/image", "2018-12-25"),
            "/posts/2018-12-25-image"
        );
    }

    #[test]
    fn test_transform_image_path_fallback_when_no_filename() {
        // Edge case: if Path can't extract a filename, use original
        assert_eq!(
            transform_image_path("..", "2019-01-01"),
            "/posts/2019-01-01-.."
        );
    }
}
