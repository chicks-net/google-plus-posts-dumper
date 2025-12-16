//! Data models for Google+ posts

#[derive(Debug, Default)]
pub struct PostData {
    pub author: String,
    pub date: String,
    pub canonical_url: String,
    pub title: String,
    pub content: String,
    pub reshare_author: Option<String>,
    pub reshare_content: Option<String>,
    pub location: Option<String>,
    pub images: Vec<String>,
    pub video_url: Option<String>,
    pub links: Vec<(String, String)>, // (url, title)
    pub visibility: String,
    pub plus_ones: Vec<String>,
    pub comments: Vec<Comment>,
}

#[derive(Debug)]
pub struct Comment {
    pub author: String,
    pub date: String,
    pub content: String,
}
