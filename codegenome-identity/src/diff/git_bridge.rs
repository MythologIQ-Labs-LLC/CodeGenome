use std::path::Path;

use crate::diff::types::{DiffStatus, OwnedDiff, OwnedDiffFile, OwnedHunk};

/// Extract OwnedDiff from a git repository between two refs.
/// If `from` is None, defaults to HEAD.
/// If `to` is None, diffs against the working tree.
pub fn git_diff(
    repo_path: &Path,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<OwnedDiff, String> {
    let repo =
        git2::Repository::open(repo_path).map_err(|e| format!("Failed to open repo: {e}"))?;

    let from_tree = resolve_tree(&repo, from.unwrap_or("HEAD"))?;

    let diff = if let Some(to_ref) = to {
        let to_tree = resolve_tree(&repo, to_ref)?;
        repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)
    } else {
        repo.diff_tree_to_workdir_with_index(Some(&from_tree), None)
    }
    .map_err(|e| format!("Failed to compute diff: {e}"))?;

    convert_diff(&diff)
}

fn resolve_tree<'a>(repo: &'a git2::Repository, refspec: &str) -> Result<git2::Tree<'a>, String> {
    let obj = repo
        .revparse_single(refspec)
        .map_err(|e| format!("Failed to resolve '{refspec}': {e}"))?;
    let commit = obj
        .peel_to_commit()
        .map_err(|e| format!("'{refspec}' is not a commit: {e}"))?;
    commit
        .tree()
        .map_err(|e| format!("Failed to get tree: {e}"))
}

fn convert_diff(diff: &git2::Diff) -> Result<OwnedDiff, String> {
    let mut files = Vec::new();

    for delta_idx in 0..diff.deltas().len() {
        let delta = diff.get_delta(delta_idx).unwrap();
        let path = delta
            .new_file()
            .path()
            .unwrap_or(Path::new("unknown"))
            .to_path_buf();
        let status = map_delta_status(delta.status());

        let mut hunks = Vec::new();
        let patch = git2::Patch::from_diff(diff, delta_idx)
            .map_err(|e| format!("Failed to get patch: {e}"))?;

        if let Some(ref patch) = patch {
            for hunk_idx in 0..patch.num_hunks() {
                let (hunk, _) = patch
                    .hunk(hunk_idx)
                    .map_err(|e| format!("Hunk error: {e}"))?;
                hunks.push(OwnedHunk {
                    new_start: hunk.new_start(),
                    new_lines: hunk.new_lines(),
                    old_start: hunk.old_start(),
                    old_lines: hunk.old_lines(),
                });
            }
        }

        files.push(OwnedDiffFile {
            path,
            status,
            hunks,
        });
    }

    Ok(OwnedDiff { files })
}

fn map_delta_status(status: git2::Delta) -> DiffStatus {
    match status {
        git2::Delta::Added => DiffStatus::Added,
        git2::Delta::Deleted => DiffStatus::Deleted,
        git2::Delta::Renamed => DiffStatus::Renamed,
        _ => DiffStatus::Modified,
    }
}
