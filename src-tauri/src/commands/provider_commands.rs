use crate::models::provider::Provider;
use crate::services::ProviderService;
use crate::utils::AppResult;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn list_providers(
    service: State<'_, Arc<ProviderService>>,
) -> AppResult<Vec<Provider>> {
    Ok(service.list_providers())
}

#[tauri::command]
pub fn get_provider(
    id: String,
    service: State<'_, Arc<ProviderService>>,
) -> AppResult<Option<Provider>> {
    Ok(service.get_provider(&id))
}

#[tauri::command]
pub fn get_default_provider(
    service: State<'_, Arc<ProviderService>>,
) -> AppResult<Option<Provider>> {
    Ok(service.get_default_provider())
}

#[tauri::command]
pub fn add_provider(
    provider: Provider,
    service: State<'_, Arc<ProviderService>>,
) -> AppResult<()> {
    Ok(service.add_provider(provider)?)
}

#[tauri::command]
pub fn update_provider(
    provider: Provider,
    service: State<'_, Arc<ProviderService>>,
) -> AppResult<()> {
    Ok(service.update_provider(provider)?)
}

#[tauri::command]
pub fn remove_provider(
    id: String,
    service: State<'_, Arc<ProviderService>>,
) -> AppResult<()> {
    Ok(service.remove_provider(&id)?)
}

#[tauri::command]
pub fn set_default_provider(
    id: String,
    service: State<'_, Arc<ProviderService>>,
) -> AppResult<()> {
    Ok(service.set_default(&id)?)
}

/// 配置目录信息
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDirInfo {
    pub path: String,
    pub has_settings: bool,
    pub has_credentials: bool,
    pub settings_summary: Option<String>,
    pub files: Vec<String>,
}

/// 读取 Claude Code 配置目录信息
#[tauri::command]
pub fn read_config_dir_info(path: String) -> AppResult<ConfigDirInfo> {
    let dir = Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("路径不存在或不是目录: {}", path).into());
    }

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                files.push(name.to_string());
            }
        }
    }
    files.sort();

    let has_settings = dir.join("settings.json").is_file();
    let has_credentials = dir.join(".credentials.json").is_file();

    let settings_summary = if has_settings {
        std::fs::read_to_string(dir.join("settings.json"))
            .ok()
            .and_then(|content| {
                serde_json::from_str::<serde_json::Value>(&content).ok()
            })
            .map(|val| {
                let mut parts = Vec::new();
                if let Some(model) = val.get("model").and_then(|v| v.as_str()) {
                    parts.push(format!("model: {}", model));
                }
                if let Some(provider) = val.get("provider").and_then(|v| v.as_str()) {
                    parts.push(format!("provider: {}", provider));
                }
                if parts.is_empty() {
                    let keys: Vec<String> = val.as_object()
                        .map(|obj| obj.keys().take(5).cloned().collect())
                        .unwrap_or_default();
                    format!("keys: {}", keys.join(", "))
                } else {
                    parts.join(", ")
                }
            })
    } else {
        None
    };

    Ok(ConfigDirInfo {
        path,
        has_settings,
        has_credentials,
        settings_summary,
        files,
    })
}

/// 在系统文件管理器中打开路径
#[tauri::command]
pub fn open_path_in_explorer(app: AppHandle, path: String) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| e.to_string())?;
    Ok(())
}
