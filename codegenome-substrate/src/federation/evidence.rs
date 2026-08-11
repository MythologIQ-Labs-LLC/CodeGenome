use std::collections::HashMap;
use std::path::Path;

use crate::federation::config::WorkspaceConfig;

pub fn dependency_edges(cfg: &WorkspaceConfig) -> Vec<(String, String, String)> {
    let by_pkg: HashMap<_, _> = cfg
        .repositories
        .iter()
        .filter_map(|repo| package_and_deps(&repo.source_dir).ok())
        .collect();
    cfg.repositories
        .iter()
        .flat_map(|repo| {
            let source = repo.name.clone();
            let deps = by_pkg.get(&source).cloned().unwrap_or_default();
            deps.into_iter()
                .filter(|dep| by_pkg.contains_key(dep))
                .map(move |dep| (source.clone(), dep, "cargo_dependency".into()))
        })
        .collect()
}

pub fn identity_edges(
    repositories: &[(String, Vec<codegenome_identity::graph::node::Node>)],
) -> Vec<(String, String, String)> {
    let mut seen: HashMap<codegenome_identity::identity::UorAddress, Vec<String>> = HashMap::new();
    for (repo, nodes) in repositories {
        for node in nodes {
            seen.entry(node.address).or_default().push(repo.clone());
        }
    }
    seen.into_values()
        .flat_map(|repos| {
            let mut out = Vec::new();
            for i in 0..repos.len() {
                for j in i + 1..repos.len() {
                    out.push((repos[i].clone(), repos[j].clone(), "shared_uor".into()));
                }
            }
            out
        })
        .collect()
}

/// Extract (importer, exporter) dependency pairs from workspace config.
pub fn dependency_pairs(cfg: &WorkspaceConfig) -> Vec<(String, String)> {
    let by_pkg: HashMap<String, Vec<String>> = cfg
        .repositories
        .iter()
        .filter_map(|repo| package_and_deps(&repo.source_dir).ok())
        .collect();
    cfg.repositories
        .iter()
        .flat_map(|repo| {
            let source = repo.name.clone();
            let deps = by_pkg.get(&source).cloned().unwrap_or_default();
            deps.into_iter()
                .filter(|dep| by_pkg.contains_key(dep))
                .map(move |dep| (source.clone(), dep))
        })
        .collect()
}

/// Resolve symbol-level cross-repo edges using language backends.
/// Only resolves across declared dependency boundaries.
pub fn symbol_edges(cfg: &WorkspaceConfig) -> Vec<codegenome_identity::graph::edge::Edge> {
    let languages = codegenome_identity::lang::all_languages();
    let pairs = dependency_pairs(cfg);
    let mut edges = Vec::new();

    for (importer_name, exporter_name) in &pairs {
        let Some(importer) = cfg.repositories.iter().find(|r| &r.name == importer_name) else {
            continue;
        };
        let Some(exporter) = cfg.repositories.iter().find(|r| &r.name == exporter_name) else {
            continue;
        };

        let importer_files = collect_files(&importer.source_dir);
        let exporter_files = collect_files(&exporter.source_dir);

        for lang in &languages {
            let exports = crate::federation::symbol_resolve::build_export_table(
                lang.as_ref(),
                &exporter_files,
            );
            let resolved = crate::federation::symbol_resolve::resolve_cross_repo(
                lang.as_ref(),
                &importer_files,
                &exports,
            );
            edges.extend(resolved);
        }
    }
    edges
}

fn collect_files(dir: &Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let supported = codegenome_identity::lang::detect::supported_extensions();
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_files(&path));
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| supported.contains(&e))
        {
            if let Ok(content) = std::fs::read(&path) {
                files.push((path, content));
            }
        }
    }
    files
}

fn package_and_deps(root: &Path) -> Result<(String, Vec<String>), String> {
    let path = root.join("Cargo.toml");
    let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value: toml::Value = toml::from_str(&data).map_err(|e| e.to_string())?;
    let package = value
        .get("package")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing package.name".to_string())?;
    let deps = value
        .get("dependencies")
        .and_then(|v| v.as_table())
        .map(|t| t.keys().cloned().collect())
        .unwrap_or_default();
    Ok((package.into(), deps))
}
