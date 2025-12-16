//! Library for parsing Google+ Takeout HTML files and converting to Markdown

pub mod dom;
pub mod markdown;
pub mod models;
pub mod parser;
pub mod utils;

// Re-export main types and functions for convenient access
pub use markdown::generate_markdown;
pub use models::{Comment, PostData};
pub use parser::extract_post_data;
pub use utils::format_filename_date;
