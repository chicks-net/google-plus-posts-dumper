
#[macro_use]
extern crate html5ever;
extern crate markup5ever_rcdom as rcdom;

use std::env;
use std::fs::File;
use std::path::Path;
use std::io;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use rcdom::{Handle, NodeData, RcDom};

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

    // find posts directory
    let posts_path = Path::new(base_path).join("Google+ Stream/Posts");
    let posts_path_string = posts_path.to_str().unwrap();
    assert_dir(&posts_path);
    println!("Posts are in {posts_path_string:?}");

    // Loop through html files
    let mut post_pattern: String = (&posts_path_string).to_string();
    post_pattern.push_str("/*.html");
    println!("Debug: {post_pattern}");
    for entry in glob(&post_pattern).expect("Failed to glob") {
        match entry {
            Ok(path) => process_file(&path.display().to_string(), dest_path_arg),
            Err(e) => println!("{:?}", e),
        }
    }
}

fn assert_dir(dir_path: &Path) {
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());
}

fn process_file(file_name: &str, dest_dir: &str) {
    let file_path = Path::new(file_name);
    assert!(file_path.exists());
    assert!(file_path.is_file());

    println!("processing {:?}",file_name);
    println!("\tinto {:?}",dest_dir);

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file_handle = match File::open(&file_path) {
        Err(why) => panic!("couldn't open {}: {}", file_name, why),
        Ok(file_handle) => file_handle,
    };
}
