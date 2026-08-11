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
