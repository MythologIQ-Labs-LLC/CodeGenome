use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Detect language from file extension.
pub fn detect_language(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some("rust"),
        "ts" => Some("typescript"),
        "tsx" => Some("typescript-tsx"),
        "py" => Some("python"),
        _ => None,
    }
}

/// Group files by detected language. Unsupported files are dropped.
pub fn group_by_language(
    files: &[(PathBuf, Vec<u8>)],
) -> HashMap<&'static str, Vec<(PathBuf, Vec<u8>)>> {
    let mut groups: HashMap<&'static str, Vec<(PathBuf, Vec<u8>)>> = HashMap::new();
    for (path, content) in files {
        if let Some(lang) = detect_language(path) {
            groups
                .entry(lang)
                .or_default()
                .push((path.clone(), content.clone()));
        }
    }
    groups
}

/// All file extensions supported by built-in backends.
pub fn supported_extensions() -> &'static [&'static str] {
    &["rs", "ts", "tsx", "py"]
}

/// Directories that must never be walked during source collection:
/// hidden directories (VCS, editor state) and vendored/generated trees
/// whose contents would pollute the graph with third-party symbols.
pub fn is_excluded_dir(dir_name: &str) -> bool {
    dir_name.starts_with('.')
        || matches!(
            dir_name,
            "target" | "node_modules" | "__pycache__" | "venv" | "vendor"
        )
}
