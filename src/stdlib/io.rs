use crate::core::value::{NativeFn, Value};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::rc::Rc;

pub fn create_io_module() -> Value {
    let mut map = IndexMap::new();

    // print
    let print_fn: NativeFn = Rc::new(|args: &[Value]| {
        let text = args
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        println!("{}", text);
        Ok(Value::Null)
    });
    map.insert("print".to_string(), Value::NativeFunction("print".to_string(), print_fn));

    // input(prompt)
    let input_fn: NativeFn = Rc::new(|args: &[Value]| {
        if let Some(prompt) = args.first() {
            print!("{}", prompt);
            let _ = io::stdout().flush();
        }
        let mut line = String::new();
        io::stdin().read_line(&mut line).map_err(|e| e.to_string())?;
        let trimmed = line.trim();

        // Autocast like Python NovaScript: int -> float -> bool -> string
        if let Ok(i) = trimmed.parse::<i64>() {
            return Ok(Value::Int(i));
        }
        if let Ok(f) = trimmed.parse::<f64>() {
            return Ok(Value::Float(f));
        }
        if trimmed.eq_ignore_ascii_case("true") {
            return Ok(Value::Bool(true));
        }
        if trimmed.eq_ignore_ascii_case("false") {
            return Ok(Value::Bool(false));
        }

        Ok(Value::String(trimmed.to_string()))
    });
    map.insert("input".to_string(), Value::NativeFunction("input".to_string(), input_fn));

    // read_file(path)
    let read_file_fn: NativeFn = Rc::new(|args: &[Value]| {
        let path = args.first().ok_or("Expected file path")?.to_string();
        let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
        Ok(Value::String(content))
    });
    map.insert("read_file".to_string(), Value::NativeFunction("read_file".to_string(), read_file_fn));

    // write_file(path, content)
    let write_file_fn: NativeFn = Rc::new(|args: &[Value]| {
        let path = args.first().ok_or("Expected file path")?.to_string();
        let content = args.get(1).map(|v| v.to_string()).unwrap_or_default();
        fs::write(&path, content).map_err(|e| format!("Failed to write file '{}': {}", path, e))?;
        Ok(Value::Bool(true))
    });
    map.insert("write_file".to_string(), Value::NativeFunction("write_file".to_string(), write_file_fn));

    // remove_file(path) / delete_file(path)
    let remove_file_fn: NativeFn = Rc::new(|args: &[Value]| {
        let path = args.first().ok_or("Expected file path")?.to_string();
        fs::remove_file(&path).map_err(|e| format!("Failed to delete file '{}': {}", path, e))?;
        Ok(Value::Bool(true))
    });
    map.insert("remove_file".to_string(), Value::NativeFunction("remove_file".to_string(), remove_file_fn.clone()));
    map.insert("delete_file".to_string(), Value::NativeFunction("delete_file".to_string(), remove_file_fn));

    // exists(path)
    let exists_fn: NativeFn = Rc::new(|args: &[Value]| {
        let path = args.first().ok_or("Expected file path")?.to_string();
        Ok(Value::Bool(Path::new(&path).exists()))
    });
    map.insert("exists".to_string(), Value::NativeFunction("exists".to_string(), exists_fn));

    // browse(url)
    let browse_fn: NativeFn = Rc::new(|args: &[Value]| {
        let target = args.first().ok_or("Expected URL or file path")?.to_string();
        println!("[Info] Открытие в браузере: {}", target);
        crate::stdlib::game2d::open_browser(&target);
        Ok(Value::Null)
    });
    map.insert("browse".to_string(), Value::NativeFunction("browse".to_string(), browse_fn));

    Value::Map(Rc::new(RefCell::new(map)))
}
