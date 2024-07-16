use std::env;
use std::path::Path;

fn main() {
    // get directory argument and verify that it is actually a directory
    let args: Vec<String> = env::args().collect();
    // dbg!(args);
    let base_path = &args[1];
    assert_eq!(Path::new(base_path).exists(), true);
    assert_eq!(Path::new(base_path).is_dir(), true);
    println!("Starting with {base_path}");

    // find posts directory
    let posts_path = Path::new(base_path).join("Google+ Stream/Posts");
    let posts_path_string = posts_path.to_str();
    assert_eq!(posts_path.exists(), true);
    assert_eq!(posts_path.is_dir(), true);
    println!("Posts are in {posts_path_string:?}");
}
