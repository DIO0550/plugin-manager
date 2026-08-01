//! plm pack コマンド
//!
//! コンポーネントまたはプラグインを配布用 ZIP にパッケージ化する。

use crate::parser::frontmatter::parse_frontmatter;
use crate::plugin::meta::manifest_resolve::{has_manifest, resolve_manifest_path};
use crate::plugin::PluginManifest;
use clap::Parser;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[derive(Debug, Parser)]
pub struct Args {
    /// Path to the component or plugin directory to package
    pub path: String,
}

/// Skill frontmatter の最低限フィールド（バリデーション用）。
#[derive(Debug, Default, Deserialize)]
struct SkillFrontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm pack`.
pub async fn run(args: Args) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let source = PathBuf::from(&args.path);
    let source = if source.is_absolute() {
        source
    } else {
        cwd.join(source)
    };

    println!("📦 Packaging {}...", display_name(&source));
    let zip_path = pack_path(&source, &cwd)?;
    println!(
        "✅ Created {}",
        zip_path.file_name().unwrap().to_string_lossy()
    );
    print_contents(&zip_path)?;
    Ok(())
}

/// パッケージ化対象の種別。
enum PackKind {
    Plugin(PluginManifest),
    Skill,
}

/// `source` ディレクトリを `output_dir/<name>.zip` にパッケージ化する。
///
/// # Arguments
///
/// * `source` - パッケージ化するディレクトリ
/// * `output_dir` - ZIP の出力先ディレクトリ（通常は CWD）
pub(crate) fn pack_path(source: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    if !source.exists() {
        return Err(format!("Path does not exist: {}", source.display()));
    }
    if !source.is_dir() {
        return Err(format!("Path must be a directory: {}", source.display()));
    }

    let kind = detect_pack_kind(source)?;
    validate_pack(source, &kind)?;
    let package_name = resolve_package_name(source, &kind)?;

    let zip_path = output_dir.join(format!("{package_name}.zip"));
    if zip_path.exists() {
        return Err(format!("Output already exists: {}", zip_path.display()));
    }

    write_zip(source, &zip_path)?;
    Ok(zip_path)
}

fn detect_pack_kind(source: &Path) -> Result<PackKind, String> {
    if has_manifest(source) {
        Ok(PackKind::Plugin(load_plugin_manifest(source)?))
    } else if source.join("SKILL.md").is_file() {
        Ok(PackKind::Skill)
    } else {
        Err(
            "Unrecognized package: expected a plugin (plugin.json) or a skill directory (SKILL.md)"
                .to_string(),
        )
    }
}

fn load_plugin_manifest(source: &Path) -> Result<PluginManifest, String> {
    let manifest_path =
        resolve_manifest_path(source).ok_or_else(|| "plugin.json not found".to_string())?;
    PluginManifest::load(&manifest_path).map_err(|e| e.to_string())
}

fn validate_pack(source: &Path, kind: &PackKind) -> Result<(), String> {
    match kind {
        PackKind::Plugin(manifest) => validate_plugin(source, manifest),
        PackKind::Skill => validate_skill_file(&source.join("SKILL.md")),
    }
}

fn resolve_package_name(source: &Path, kind: &PackKind) -> Result<String, String> {
    match kind {
        PackKind::Plugin(manifest) => Ok(manifest.name.clone()),
        PackKind::Skill => source
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Could not determine skill directory name".to_string()),
    }
}

fn validate_plugin(source: &Path, manifest: &PluginManifest) -> Result<(), String> {
    if manifest.name.trim().is_empty() {
        return Err("plugin.json 'name' must not be empty".to_string());
    }
    if manifest.version.trim().is_empty() {
        return Err("plugin.json 'version' must not be empty".to_string());
    }

    // 宣言パス上の skill があれば frontmatter を軽く検証
    let skills_dir = manifest.skills_dir(source);
    if skills_dir.is_dir() {
        for entry in fs::read_dir(&skills_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let skill_md = entry.path().join("SKILL.md");
            if skill_md.is_file() {
                validate_skill_file(&skill_md)?;
            }
        }
    }

    Ok(())
}

fn validate_skill_file(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed = parse_frontmatter::<SkillFrontmatter>(&content)
        .map_err(|e| format!("Invalid YAML frontmatter in {}: {e}", path.display()))?;

    let Some(fm) = parsed.frontmatter else {
        return Err(format!("Missing YAML frontmatter in {}", path.display()));
    };

    let name = fm.name.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if name.is_none() {
        return Err(format!(
            "Frontmatter 'name' is required in {}",
            path.display()
        ));
    }

    let description = fm
        .description
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if description.is_none() {
        return Err(format!(
            "Frontmatter 'description' is required in {}",
            path.display()
        ));
    }

    Ok(())
}

fn write_zip(source: &Path, zip_path: &Path) -> Result<(), String> {
    let file = File::create(zip_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(source)
        .into_iter()
        .filter_entry(|e| !should_exclude(source, e.path()))
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path == source {
            continue;
        }

        // シンボリックリンクは含めない
        let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }

        let rel = path.strip_prefix(source).map_err(|e| e.to_string())?;
        let name = rel.to_string_lossy().replace('\\', "/");
        if name.is_empty() {
            continue;
        }

        if meta.is_dir() {
            let dir_name = if name.ends_with('/') {
                name
            } else {
                format!("{name}/")
            };
            zip.add_directory(dir_name, options)
                .map_err(|e| e.to_string())?;
        } else if meta.is_file() {
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            let bytes = fs::read(path).map_err(|e| e.to_string())?;
            zip.write_all(&bytes).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn should_exclude(root: &Path, path: &Path) -> bool {
    if path == root {
        return false;
    }
    let Ok(rel) = path.strip_prefix(root) else {
        return true;
    };
    rel.components().any(|c| {
        let s = c.as_os_str();
        s == ".git" || s == ".plm-meta.json"
    })
}

fn display_name(source: &Path) -> String {
    source
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| source.to_str().unwrap_or("."))
        .to_string()
}

fn print_contents(zip_path: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.by_index(i).ok().map(|e| e.name().to_string()))
        .collect();
    names.sort();
    println!("   Contents:");
    for name in names {
        println!("   └── {name}");
    }
    Ok(())
}

#[cfg(test)]
#[path = "pack_test.rs"]
mod tests;
