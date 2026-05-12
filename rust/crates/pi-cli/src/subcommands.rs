//! 扩展管理子命令实现。
//!
//! 支持 install/remove/update/list 四个扩展生命周期操作。
//! 扩展配置存储在 `~/.piso/settings.json` 的 `extensions` 数组中。

use crate::config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 扩展配置条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionEntry {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub enabled: bool,
}

/// settings.json 的扩展相关部分。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub extensions: Vec<ExtensionEntry>,
}

impl Settings {
    /// 从 `~/.piso/settings.json` 加载。
    fn load() -> Result<Self> {
        let path = settings_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// 写回 `~/.piso/settings.json`。
    fn save(&self) -> Result<()> {
        let path = settings_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))
    }

    /// 查找扩展索引。
    fn find_index(&self, name_or_source: &str) -> Option<usize> {
        self.extensions.iter().position(|e| {
            e.name == name_or_source
                || e.source == name_or_source
                || e.source.ends_with(&format!("/{}", name_or_source))
                || e.source.ends_with(&format!("\\{}", name_or_source))
        })
    }
}

/// settings.json 路径。
fn settings_path() -> Result<PathBuf> {
    let config_dir = config::config_dir().unwrap_or_else(|| PathBuf::from(".piso"));
    Ok(config_dir.join("settings.json"))
}

/// 安装扩展。
pub fn install(source: &str, local: bool) -> Result<()> {
    let mut settings = Settings::load()?;

    // 检查是否已安装
    if settings.find_index(source).is_some() {
        println!("Extension '{}' is already installed.", source);
        return Ok(());
    }

    let name = derive_extension_name(source);
    let canonical_source = if local {
        // 本地路径：规范化为绝对路径
        fs::canonicalize(source)
            .with_context(|| format!("Cannot resolve path: {source}"))?
            .to_string_lossy()
            .to_string()
    } else {
        source.to_string()
    };

    settings.extensions.push(ExtensionEntry {
        name: name.clone(),
        source: canonical_source,
        enabled: true,
    });

    settings.save()?;
    println!("Installed extension '{}' from {}", name, source);
    Ok(())
}

/// 移除扩展。
pub fn remove(source: &str) -> Result<()> {
    let mut settings = Settings::load()?;

    let idx = settings
        .find_index(source)
        .ok_or_else(|| anyhow::anyhow!("Extension '{}' not found", source))?;

    let removed = settings.extensions.remove(idx);
    settings.save()?;
    println!("Removed extension '{}' ({})", removed.name, removed.source);
    Ok(())
}

/// 更新扩展或 piso 自身。
pub fn update(target: Option<&str>) -> Result<()> {
    match target {
        Some("self") | Some("piso") => {
            println!("piso update is not yet implemented.");
            println!("Download the latest binary from the releases page.");
        }
        Some(name) => {
            let settings = Settings::load()?;
            match settings.find_index(name) {
                Some(idx) => {
                    let ext = &settings.extensions[idx];
                    println!("Updating extension '{}' from {} ...", ext.name, ext.source);
                    println!("Extension update requires manual re-download.");
                }
                None => {
                    println!("Extension '{}' not found.", name);
                }
            }
        }
        None => {
            let settings = Settings::load()?;
            if settings.extensions.is_empty() {
                println!("No extensions installed.");
            } else {
                for ext in &settings.extensions {
                    println!("  {} ({})", ext.name, ext.source);
                }
                println!("Use 'piso update <name>' to update a specific extension.");
            }
        }
    }
    Ok(())
}

/// 列出已安装扩展。
pub fn list() -> Result<()> {
    let settings = Settings::load()?;

    if settings.extensions.is_empty() {
        println!("No extensions installed.");
        println!("Use 'piso install <source>' to install one.");
        return Ok(());
    }

    println!("Installed extensions:");
    for ext in &settings.extensions {
        let status = if ext.enabled { "enabled" } else { "disabled" };
        println!("  {} ({}) [{}]", ext.name, ext.source, status);
    }
    println!("\n{} extension(s) total.", settings.extensions.len());
    Ok(())
}

/// 从 source 路径/URL 推断扩展名称。
fn derive_extension_name(source: &str) -> String {
    let path = std::path::Path::new(source);
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // 去掉 .rs / .js 后缀
    let name = file_name.trim_end_matches(".rs").trim_end_matches(".js");

    // 如果是目录名，直接用
    if name.is_empty() || name == "." {
        return "unnamed-extension".to_string();
    }

    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_name_from_file() {
        assert_eq!(derive_extension_name("./my-ext.rs"), "my-ext");
        assert_eq!(derive_extension_name("/path/to/foo.js"), "foo");
        assert_eq!(derive_extension_name("bar"), "bar");
    }

    #[test]
    fn derive_name_from_url() {
        assert_eq!(
            derive_extension_name("https://example.com/ext/hello.rs"),
            "hello"
        );
    }

    #[test]
    fn settings_find_index() {
        let settings = Settings {
            extensions: vec![
                ExtensionEntry {
                    name: "foo".to_string(),
                    source: "/path/foo.rs".to_string(),
                    enabled: true,
                },
                ExtensionEntry {
                    name: "bar".to_string(),
                    source: "https://x.com/bar.rs".to_string(),
                    enabled: true,
                },
            ],
        };

        assert_eq!(settings.find_index("foo"), Some(0));
        assert_eq!(settings.find_index("/path/foo.rs"), Some(0));
        assert_eq!(settings.find_index("bar"), Some(1));
        assert_eq!(settings.find_index("baz"), None);
    }
}
