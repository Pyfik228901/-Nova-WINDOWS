use std::env;
use std::fs;
use std::path::PathBuf;

pub const PACKAGE_JSON: &str = include_str!("embedded/package.json");
pub const LANGUAGE_CONFIG_JSON: &str = include_str!("embedded/language-configuration.json");
pub const TM_LANGUAGE_JSON: &str = include_str!("embedded/ns.tmLanguage.json");
pub const SNIPPETS_JSON: &str = include_str!("embedded/snippets.json");
pub const EXTENSION_JS: &str = include_str!("embedded/extension.js");
pub const ICON_THEME_JSON: &str = include_str!("embedded/ns-icon-theme.json");
pub const LOGO_SVG: &str = include_str!("../../assets/logo.svg");

pub fn install_vscode_extension() -> Result<(), String> {
    let extensions_dir = get_vscode_extensions_dir()
        .ok_or_else(|| "Не удалось определить путь к директории расширений VS Code".to_string())?;

    // Cleanup legacy directory if it was created without publisher prefix
    let legacy_dir = extensions_dir.join("ns-lang-support-1.0.0");
    if legacy_dir.exists() {
        let _ = fs::remove_dir_all(&legacy_dir);
    }

    let target_dir = extensions_dir.join("m1oky.ns-lang-support-1.0.0");
    let icons_dir = target_dir.join("icons");
    let syntaxes_dir = target_dir.join("syntaxes");
    let snippets_dir = target_dir.join("snippets");

    fs::create_dir_all(&icons_dir)
        .map_err(|e| format!("Ошибка создания папки {}: {}", icons_dir.display(), e))?;
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
    fs::write(icons_dir.join("logo.svg"), LOGO_SVG)
        .map_err(|e| format!("Ошибка записи logo.svg: {}", e))?;
    fs::write(icons_dir.join("ns-icon-theme.json"), ICON_THEME_JSON)
        .map_err(|e| format!("Ошибка записи ns-icon-theme.json: {}", e))?;
    fs::write(syntaxes_dir.join("ns.tmLanguage.json"), TM_LANGUAGE_JSON)
        .map_err(|e| format!("Ошибка записи ns.tmLanguage.json: {}", e))?;
    fs::write(snippets_dir.join("snippets.json"), SNIPPETS_JSON)
        .map_err(|e| format!("Ошибка записи snippets.json: {}", e))?;

    println!();
    println!("\x1b[32m[OK] Расширение N&S успешно установлено в VS Code.\x1b[0m");
    println!("  ID расширения: m1oky.ns-lang-support");
    println!("  Путь: {}", target_dir.display());
    println!("  Включено: подсветка синтаксиса, автодополнение методов и ключевых слов, сниппеты, иконки файлов .ns.");
    println!("  Перезапустите VS Code или выполните Developer: Reload Window (Ctrl+R), чтобы применить изменения.");
    println!();

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
