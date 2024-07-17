use std::env;
use std::path::Path;
use glob::glob;

fn main() {
    // get directory argument and verify that it is actually a directory
    let args: Vec<String> = env::args().collect();
    // dbg!(args);
    let base_path = &args[1];
    assert!(Path::new(base_path).exists());
    assert!(Path::new(base_path).is_dir());
    println!("Starting with {base_path}");

    // find posts directory
    let posts_path = Path::new(base_path).join("Google+ Stream/Posts");
    let posts_path_string = posts_path.to_str().unwrap();
    assert!(posts_path.exists());
    assert!(posts_path.is_dir());
    println!("Posts are in {posts_path_string:?}");

    // Loop through html files
    let mut post_pattern: String = (&posts_path_string).to_string();
    post_pattern.push_str("/*.html");
    println!("Debug: {post_pattern}");
    for entry in glob(&post_pattern).expect("Failed to glob") {
        match entry {
            Ok(path) => println!("{:?}", path.display()),
            Err(e) => println!("{:?}", e),
        }
    }
}
