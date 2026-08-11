use std::path::Path;

use crate::diff::git_bridge::git_diff;
use crate::diff::DiffStatus;

fn init_repo(dir: &Path) {
    let repo = git2::Repository::init(dir).unwrap();
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "test").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();

    // Create initial file and commit
    let file = dir.join("main.rs");
    std::fs::write(&file, "fn main() {}\n").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("main.rs")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .unwrap();
}

#[test]
fn modified_file_produces_diff() {
    let dir = std::env::temp_dir().join("cg_git_test_mod");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    init_repo(&dir);

    // Modify the file
    std::fs::write(
        dir.join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    let diff = git_diff(&dir, None, None).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(diff.files.len(), 1);
    assert_eq!(diff.files[0].status, DiffStatus::Modified);
    assert!(!diff.files[0].hunks.is_empty());
}

#[test]
fn no_changes_produces_empty_diff() {
    let dir = std::env::temp_dir().join("cg_git_test_empty");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    init_repo(&dir);

    let diff = git_diff(&dir, None, None).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(diff.files.is_empty());
}

#[test]
fn hunk_lines_match_modification() {
    let dir = std::env::temp_dir().join("cg_git_test_hunk");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    init_repo(&dir);

    // Modify line 1
    std::fs::write(dir.join("main.rs"), "fn main() { /* changed */ }\n").unwrap();

    let diff = git_diff(&dir, None, None).unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(diff.files.len(), 1);
    let hunk = &diff.files[0].hunks[0];
    assert_eq!(hunk.new_start, 1, "Change should start at line 1");
}
