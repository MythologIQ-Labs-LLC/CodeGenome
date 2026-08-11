use std::path::PathBuf;

use crate::diff::detect_changes;
use crate::diff::{DiffStatus, OwnedDiff, OwnedDiffFile, OwnedHunk};
use crate::graph::overlay::Overlay;
use crate::overlay::syntax::parse_rust_files;

fn make_diff_touching_line(path: &str, line: u32) -> OwnedDiff {
    OwnedDiff {
        files: vec![OwnedDiffFile {
            path: PathBuf::from(path),
            status: DiffStatus::Modified,
            hunks: vec![OwnedHunk {
                new_start: line,
                new_lines: 3,
                old_start: line,
                old_lines: 3,
            }],
        }],
    }
}

#[test]
fn impact_propagation_from_diff() {
    let files = crate::tests::self_index::load_own_source();
    let overlay = parse_rust_files(&files);

    // Create a diff touching line 5 of some file
    let first_file = &files[0].0;
    let diff = make_diff_touching_line(&first_file.to_string_lossy(), 5);

    let changeset = detect_changes(&diff, &[&overlay as &dyn Overlay]);

    // Impact map should be non-empty if any symbol spans line 5
    if !changeset.changed_nodes.is_empty() {
        assert!(!changeset.impact.is_empty());
    }
}

#[test]
fn empty_diff_produces_empty_changeset() {
    let files = crate::tests::self_index::load_own_source();
    let overlay = parse_rust_files(&files);

    let diff = OwnedDiff { files: vec![] };
    let changeset = detect_changes(&diff, &[&overlay as &dyn Overlay]);

    assert!(changeset.changed_nodes.is_empty());
    assert!(changeset.impact.is_empty());
    assert!(changeset.staleness.is_empty());
}
