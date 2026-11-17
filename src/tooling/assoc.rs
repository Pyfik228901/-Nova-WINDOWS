use std::env;
use std::path::PathBuf;
use std::process::Command;

pub fn register_associations() -> Result<(), String> {
    #[cfg(windows)]
    {
        let current_exe = env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?;
        let exe_str = current_exe.to_string_lossy().to_string();

        let icon_str = find_icon_path(&current_exe)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("{},0", exe_str));

        // 1. .ns extension
        run_reg(&["add", "HKCU\\Software\\Classes\\.ns", "/ve", "/d", "NS.Script", "/f"])?;
        run_reg(&["add", "HKCU\\Software\\Classes\\.ns", "/v", "Content Type", "/d", "text/plain", "/f"])?;
        run_reg(&["add", "HKCU\\Software\\Classes\\.ns\\OpenWithProgids", "/v", "NS.Script", "/t", "REG_NONE", "/d", "", "/f"])?;

        // 2. .nova extension
        run_reg(&["add", "HKCU\\Software\\Classes\\.nova", "/ve", "/d", "NS.Script", "/f"])?;
        run_reg(&["add", "HKCU\\Software\\Classes\\.nova", "/v", "Content Type", "/d", "text/plain", "/f"])?;
        run_reg(&["add", "HKCU\\Software\\Classes\\.nova\\OpenWithProgids", "/v", "NS.Script", "/t", "REG_NONE", "/d", "", "/f"])?;

        // 3. NS.Script ProgID
        run_reg(&["add", "HKCU\\Software\\Classes\\NS.Script", "/ve", "/d", "Исходный файл N&S (Nova Script)", "/f"])?;
        run_reg(&["add", "HKCU\\Software\\Classes\\NS.Script\\DefaultIcon", "/ve", "/d", &format!("{},0", icon_str), "/f"])?;
        run_reg(&["add", "HKCU\\Software\\Classes\\NS.Script\\shell\\open\\command", "/ve", "/d", &format!("\"{}\" \"%1\"", exe_str), "/f"])?;

        // 4. Notify Windows Shell of association change
        notify_shell();

        println!();
        println!("\x1b[32m[OK] Ассоциации файлов .ns и .nova и иконка N&S успешно зарегистрированы в Windows.\x1b[0m");
        println!("  Исполняемый файл: {}", exe_str);
        println!("  Файл иконки: {}", icon_str);
        println!("  Теперь файлы .ns и .nova отображаются с логотипом N&S в Проводнике.");
        println!();

        Ok(())
    }

    #[cfg(not(windows))]
    {
        println!("[Info] Регистрация ассоциаций файлов через реестр поддерживается только на Windows.");
        Ok(())
    }
}

fn find_icon_path(current_exe: &PathBuf) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(parent) = current_exe.parent() {
        candidates.push(parent.join("assets").join("icon.ico"));
        candidates.push(parent.join("icon.ico"));
        candidates.push(parent.join("..").join("assets").join("icon.ico"));
        candidates.push(parent.join("..").join("..").join("assets").join("icon.ico"));
    }
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join("assets").join("icon.ico"));
        candidates.push(cwd.join("icon.ico"));
    }
    for candidate in &candidates {
        if candidate.exists() {
            let path = if let Ok(c) = candidate.canonicalize() { c } else { candidate.clone() };
            let s = path.to_string_lossy().to_string();
            let clean = s.strip_prefix(r"\\?\").unwrap_or(&s).to_string();
            return Some(PathBuf::from(clean));
        }
    }
    None
}

#[cfg(windows)]
fn run_reg(args: &[&str]) -> Result<(), String> {
    let output = Command::new("reg")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute 'reg': {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Registry command failed: {}", err.trim()));
    }
    Ok(())
}

#[cfg(windows)]
fn notify_shell() {
    let script = "Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class S { [DllImport(\"shell32.dll\")] public static extern void SHChangeNotify(uint w, uint u, IntPtr d1, IntPtr d2); }'; [S]::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)";
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", script])
        .output();
}
