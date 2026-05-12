//! GitHub Copilot OAuth device flow。
//!
//! 实现 GitHub device authorization flow：
//! 1. POST /login/device/code 获取 user_code
//! 2. 用户在浏览器输入 code
//! 3. 轮询 /login/oauth/access_token 获取 token
//! 4. Token 缓存到 ~/.piso/auth.json

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const CLIENT_ID: &str = "Iv1.b507a08c87ecfe98"; // Copilot CLI client ID

/// device code 响应。
#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: Option<u64>,
}

/// token 响应。
#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
}

/// 缓存的 GitHub token。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: Option<String>,
    pub provider: String,
}

/// 启动 device flow，返回 (user_code, verification_url)。
pub async fn start_device_flow() -> Result<(String, String)> {
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("scope", "copilot");

    let resp = client
        .post(GITHUB_DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await
        .context("Failed to request device code")?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Device code request failed: {}", body);
    }

    let data: DeviceCodeResponse = resp
        .json()
        .await
        .context("Failed to parse device code response")?;

    Ok((data.user_code, data.verification_uri))
}

/// 轮询等待 token。返回 access_token。
/// `timeout_secs` 是最大等待时间。
pub async fn poll_for_token(timeout_secs: u64) -> Result<String> {
    let client = reqwest::Client::new();

    // 先获取 device code
    let mut params = HashMap::new();
    params.insert("client_id", CLIENT_ID);
    params.insert("scope", "copilot");

    let resp = client
        .post(GITHUB_DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await
        .context("Failed to request device code")?;

    let data: DeviceCodeResponse = resp.json().await?;

    let interval = data.interval.unwrap_or(5);
    let device_code = data.device_code;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);

    loop {
        if std::time::Instant::now() > deadline {
            anyhow::bail!("OAuth device flow timed out after {}s", timeout_secs);
        }

        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

        let mut poll_params = HashMap::new();
        poll_params.insert("client_id", CLIENT_ID);
        poll_params.insert("device_code", &device_code);
        poll_params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");

        let resp = client
            .post(GITHUB_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&poll_params)
            .send()
            .await
            .context("Token poll request failed")?;

        let token_resp: TokenResponse = resp.json().await?;

        if let Some(token) = token_resp.access_token {
            return Ok(token);
        }

        match token_resp.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
            Some("expired_token") => anyhow::bail!("Device code expired. Please try again."),
            Some(err) => {
                let desc = token_resp.error_description.unwrap_or_default();
                anyhow::bail!("OAuth error: {} - {}", err, desc);
            }
            None => continue,
        }
    }
}

/// 保存 token 到 ~/.piso/auth.json。
pub fn save_token(token: &str, auth_path: &std::path::Path) -> Result<()> {
    let mut entries: serde_json::Value = if auth_path.exists() {
        let content = std::fs::read_to_string(auth_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // 添加 copilot token
    if let Some(obj) = entries.as_object_mut() {
        obj.insert(
            "copilot".to_string(),
            serde_json::json!({
                "access_token": token,
                "token_type": "bearer",
                "provider": "github-copilot"
            }),
        );
    }

    if let Some(parent) = auth_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(&entries)?;
    std::fs::write(auth_path, content)?;
    Ok(())
}

/// 从 ~/.piso/auth.json 读取缓存的 token。
pub fn get_cached_token(auth_path: &std::path::Path) -> Option<String> {
    if !auth_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(auth_path).ok()?;
    let entries: serde_json::Value = serde_json::from_str(&content).ok()?;
    entries
        .get("copilot")?
        .get("access_token")?
        .as_str()
        .map(|s| s.to_string())
}

/// 清除缓存的 token。
pub fn clear_token(auth_path: &std::path::Path) -> Result<()> {
    if !auth_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(auth_path)?;
    let mut entries: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(obj) = entries.as_object_mut() {
        obj.remove("copilot");
    }

    let content = serde_json::to_string_pretty(&entries)?;
    std::fs::write(auth_path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_read_token() {
        let dir = std::env::temp_dir().join("piso_test_oauth");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let auth_path = dir.join("auth.json");

        // 保存
        save_token("gho_test123", &auth_path).unwrap();

        // 读取
        let token = get_cached_token(&auth_path);
        assert_eq!(token, Some("gho_test123".to_string()));

        // 清除
        clear_token(&auth_path).unwrap();
        let token = get_cached_token(&auth_path);
        assert_eq!(token, None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_cached_token_missing_file() {
        let token = get_cached_token(std::path::Path::new("/nonexistent/auth.json"));
        assert_eq!(token, None);
    }

    #[test]
    fn save_preserves_existing_entries() {
        let dir = std::env::temp_dir().join("piso_test_oauth_preserve");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let auth_path = dir.join("auth.json");

        // 写入初始内容
        std::fs::write(&auth_path, r#"{"glm": {"apiKey": "test-key"}}"#).unwrap();

        // 保存 copilot token
        save_token("gho_copilot", &auth_path).unwrap();

        // 验证两个都存在
        let content = std::fs::read_to_string(&auth_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["glm"]["apiKey"], "test-key");
        assert_eq!(json["copilot"]["access_token"], "gho_copilot");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
