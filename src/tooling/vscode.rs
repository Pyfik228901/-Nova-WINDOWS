use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PACKAGE_JSON: &str = include_str!("embedded/package.json");
pub const LANGUAGE_CONFIG_JSON: &str = include_str!("embedded/language-configuration.json");
pub const TM_LANGUAGE_JSON: &str = include_str!("embedded/ns.tmLanguage.json");
pub const SNIPPETS_JSON: &str = include_str!("embedded/snippets.json");
pub const EXTENSION_JS: &str = include_str!("embedded/extension.js");
pub const ICON_PNG: &[u8] = include_bytes!("../../assets/icon.png");
pub const LOGO_SVG: &[u8] = include_bytes!("../../assets/logo.svg");

pub fn install_vscode_extension() -> Result<(), String> {
    let extensions_dir = get_vscode_extensions_dir()
        .ok_or_else(|| "Не удалось определить путь к директории расширений VS Code".to_string())?;

    // Cleanup legacy directory if it was created without publisher prefix
    let legacy_dir = extensions_dir.join("ns-lang-support-1.0.0");
    if legacy_dir.exists() {
        let _ = fs::remove_dir_all(&legacy_dir);
    }

    let target_dir = extensions_dir.join("m1oky.ns-lang-support-1.0.0");
    let syntaxes_dir = target_dir.join("syntaxes");
    let snippets_dir = target_dir.join("snippets");

    // Remove old target dir if exists to ensure clean state
    if target_dir.exists() {
        let _ = fs::remove_dir_all(&target_dir);
    }

    fs::create_dir_all(&syntaxes_dir)
        .map_err(|e| format!("Ошибка создания папки {}: {}", syntaxes_dir.display(), e))?;
    fs::create_dir_all(&snippets_dir)
        .map_err(|e| format!("Ошибка создания папки {}: {}", snippets_dir.display(), e))?;

    fs::write(target_dir.join("package.json"), PACKAGE_JSON)
        .map_err(|e| format!("Ошибка записи package.json: {}", e))?;
    fs::write(target_dir.join("language-configuration.json"), LANGUAGE_CONFIG_JSON)
        .map_err(|e| format!("Ошибка записи language-configuration.json: {}", e))?;
    fs::write(target_dir.join("extension.js"), EXTENSION_JS)
        .map_err(|e| format!("Ошибка записи extension.js: {}", e))?;
    fs::write(target_dir.join("icon.png"), ICON_PNG)
        .map_err(|e| format!("Ошибка записи icon.png: {}", e))?;
    fs::write(target_dir.join("logo.svg"), LOGO_SVG)
        .map_err(|e| format!("Ошибка записи logo.svg: {}", e))?;
    fs::write(syntaxes_dir.join("ns.tmLanguage.json"), TM_LANGUAGE_JSON)
        .map_err(|e| format!("Ошибка записи ns.tmLanguage.json: {}", e))?;
    fs::write(snippets_dir.join("snippets.json"), SNIPPETS_JSON)
        .map_err(|e| format!("Ошибка записи snippets.json: {}", e))?;

    // Регистрация в extensions.json
    register_in_extensions_json(&extensions_dir, &target_dir)?;

    println!();
    println!("\x1b[32m[OK] Расширение N&S успешно установлено и зарегистрировано в VS Code.\x1b[0m");
    println!("  ID расширения: m1oky.ns-lang-support");
    println!("  Версия: 1.0.0");
    println!("  Путь: {}", target_dir.display());
    println!("  Запись в extensions.json: обновлена");
    println!("  Включено: подсветка синтаксиса, автодополнение методов и ключевых слов, сниппеты, иконка языка.");
    println!("  Перезапустите VS Code или выполните Developer: Reload Window (Ctrl+R), чтобы применить изменения.");
    println!();

    Ok(())
}

fn register_in_extensions_json(extensions_dir: &Path, target_dir: &Path) -> Result<(), String> {
    let extensions_json_path = extensions_dir.join("extensions.json");

    let mut list: Vec<Value> = if extensions_json_path.exists() {
        match fs::read_to_string(&extensions_json_path) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(Value::Array(arr)) => arr,
                _ => Vec::new(),
            },
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    // Filter out previous version of m1oky.ns-lang-support
    list.retain(|item| {
        let id_match = item
            .get("identifier")
            .and_then(|id| id.get("id"))
            .and_then(|id_str| id_str.as_str())
            .map(|id_str| id_str == "m1oky.ns-lang-support")
            .unwrap_or(false);

        let loc_match = item
            .get("relativeLocation")
            .and_then(|loc| loc.as_str())
            .map(|loc_str| loc_str.starts_with("m1oky.ns-lang-support"))
            .unwrap_or(false);

        !id_match && !loc_match
    });

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(1794901500000);

    let path_str = target_dir.to_string_lossy().replace('\\', "/");
    let formatted_path = if path_str.starts_with('/') {
        path_str
    } else {
        format!("/{}", path_str)
    };

    let new_entry = json!({
        "identifier": {
            "id": "m1oky.ns-lang-support"
        },
        "version": "1.0.0",
        "location": {
            "$mid": 1,
            "path": formatted_path,
            "scheme": "file"
        },
        "relativeLocation": "m1oky.ns-lang-support-1.0.0",
        "metadata": {
            "isApplicationScoped": false,
            "isMachineScoped": false,
            "isBuiltin": false,
            "installedTimestamp": timestamp,
            "pinned": true,
            "source": "vsix",
            "publisherDisplayName": "m1oky",
            "isPreReleaseVersion": false
        }
    });

    list.push(new_entry);

    let serialized = serde_json::to_string_pretty(&list)
        .map_err(|e| format!("Ошибка сериализации extensions.json: {}", e))?;

    fs::write(&extensions_json_path, serialized)
        .map_err(|e| format!("Ошибка записи {}: {}", extensions_json_path.display(), e))?;

    Ok(())
}

fn get_vscode_extensions_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Ok(profile) = env::var("USERPROFILE") {
            let path = PathBuf::from(profile).join(".vscode").join("extensions");
            return Some(path);
        }
    }

    if let Ok(home) = env::var("HOME") {
        let path = PathBuf::from(home).join(".vscode").join("extensions");
        return Some(path);
    }

    None
}
