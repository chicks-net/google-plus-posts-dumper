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

#[macro_use]
extern crate html5ever;
extern crate markup5ever_rcdom as rcdom;

use std::env;
use std::fs::File;
use std::path::Path;

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
    walk(0, &dom.document);

    if !dom.errors.is_empty() {
        println!("\nParse errors that may not matter:");
        for err in dom.errors.iter() {
            println!("    {}", err);
        }
    }

    // Die for debugging
    panic!("at the disco");
}

/// Handle parsing an individual HTML element
fn walk(indent: usize, handle: &Handle) {
    let node = handle;
    // println!("walk() indent={}", indent);
    for _ in 0..indent {
        print!(" ");
    }
    match node.data {
        NodeData::Document => println!("#Document"),

        NodeData::Doctype {
            ref name,
            ref public_id,
            ref system_id,
        } => println!("<!DOCTYPE {} \"{}\" \"{}\">", name, public_id, system_id),

        NodeData::Text { ref contents } => {
            println!("#text: {}", contents.borrow().escape_default())
        },

        NodeData::Comment { ref contents } => println!("<!-- {} -->", contents.escape_default()),

        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            assert!(name.ns == ns!(html));
            print!("<{}", name.local);
            for attr in attrs.borrow().iter() {
                assert!(attr.name.ns == ns!());
                print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            println!(">");
        },

        NodeData::ProcessingInstruction { .. } => unreachable!(),
    }

    for child in node.children.borrow().iter() {
        walk(indent + 4, child);
    }
}
