mod print;

use clap::Parser;
use hol::hol;
use git2::Repository;
use tempfile::tempdir;
use crate::print::print_diff_line;

/// A Rust and Git aware diff command.
/// Compare two Rust code items, specified by git ref (optionally, e.g. "main"),
/// file path (e.g. "src/lib.rs"), and code item path (e.g. "my_mod::MyStruct::new").
#[derive(Parser)]
struct Args {
    /// File path, optionally prefixed with git-ref (e.g., "main:src/lib.rs" or just "src/lib.rs").
    /// The old version of the file.
    #[arg(value_parser = parse_git_file_ref)]
    old_file: GitFileRef,

    /// Path to the item to be compared (e.g. "my_mod::MyStruct::new").
    old_item_path: String,

    /// File path, optionally prefixed with git-ref (e.g., "main:src/lib.rs" or just "src/lib.rs").
    /// The new version of the file.
    #[arg(value_parser = parse_git_file_ref)]
    new_file: GitFileRef,

    /// Path to the item to be compared (e.g. "my_other_mod::MyStruct::new").
    new_item_path: String,
}

#[derive(Debug, Clone)]
struct GitFileRef {
    git_ref: Option<String>,
    file_path: String,
}

fn parse_git_file_ref(s: &str) -> Result<GitFileRef, String> {
    let res = if let Some((git_ref, file_path)) = s.split_once(':') {
        GitFileRef {
            git_ref: Some(git_ref.to_string()),
            file_path: file_path.to_string(),
        }
    } else {
        GitFileRef {
            git_ref: None,
            file_path: s.to_string(),
        }
    };
    Ok(res)
}

fn main() {
    let args = Args::parse();
    let old_item = hol(&args.old_file.file_path, args.old_file.git_ref.as_deref(), args.old_item_path).expect("Failed to read old item").expect("Old item could not be found in file.");
    let new_item = hol(&args.new_file.file_path, args.new_file.git_ref.as_deref(), args.new_item_path).expect("Failed to read new item").expect("New item could not be found in file.");
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let repo = Repository::init_bare(&temp_dir.path()).expect("Failed to create temporary git repository");
    let old_blob = repo.blob(old_item.as_bytes()).expect("Failed to create temporary git blob with just the old item");
    let old_blob = repo.find_blob(old_blob).unwrap();
    let new_blob = repo.blob(new_item.as_bytes()).expect("Failed to create temporary git blob with just the new item");
    let new_blob = repo.find_blob(new_blob).unwrap();
    repo.diff_blobs(Some(&old_blob), None, Some(&new_blob), None, None, None, None, None, Some(&mut print_diff_line)).expect("Failed to create diff");
}