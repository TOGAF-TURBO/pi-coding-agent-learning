//! 版本检查 — 启动时异步检查 crates.io 是否有新版本。
//!
//! 仅在非 offline 模式下检查。结果以 hint 形式输出到 stderr。
//! 不会阻塞启动。


/// 当前 piso 版本。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 检查 crates.io 是否有新版本。
///
/// 返回最新版本号（如果有更新）。
pub async fn check_for_update() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let resp = client
        .get("https://crates.io/api/v1/crates/piso")
        .header("User-Agent", format!("piso/{}", VERSION))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let body: serde_json::Value = resp.json().await.ok()?;
    let latest = body.get("crate")
        .and_then(|c| c.get("max_version"))
        .and_then(|v| v.as_str())?;

    if latest != VERSION {
        Some(latest.to_string())
    } else {
        None
    }
}

/// 格式化更新提示。
pub fn format_update_hint(latest: &str) -> String {
    format!(
        "piso {} available (current: {}). Update: cargo install piso",
        latest, VERSION
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        // 确保 VERSION 不是空字符串
        assert!(!VERSION.is_empty());
        // 确保版本号格式正确
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(parts.len() >= 2);
    }

    #[test]
    fn format_hint() {
        let hint = format_update_hint("99.0.0");
        assert!(hint.contains("99.0.0"));
        assert!(hint.contains(VERSION));
    }
}
