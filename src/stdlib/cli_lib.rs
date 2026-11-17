use crate::core::value::{NativeFn, Value};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_cli_module(script_args: &[String]) -> Value {
    let mut map = IndexMap::new();

    // cli.args()
    let args_val: Vec<Value> = script_args.iter().map(|s| Value::String(s.clone())).collect();
    let args_rc = Rc::new(RefCell::new(args_val));
    let args_fn: NativeFn = {
        let args_rc = args_rc.clone();
        Rc::new(move |_args: &[Value]| Ok(Value::Array(args_rc.clone())))
    };
    map.insert("args".to_string(), Value::NativeFunction("args".to_string(), args_fn));

    // Color styling helpers
    let red_fn: NativeFn = Rc::new(|args: &[Value]| {
        let text = args.first().map(|v| v.to_string()).unwrap_or_default();
        Ok(Value::String(format!("\x1b[31m{}\x1b[0m", text)))
    });
    map.insert("red".to_string(), Value::NativeFunction("red".to_string(), red_fn));

    let green_fn: NativeFn = Rc::new(|args: &[Value]| {
        let text = args.first().map(|v| v.to_string()).unwrap_or_default();
        Ok(Value::String(format!("\x1b[32m{}\x1b[0m", text)))
    });
    map.insert("green".to_string(), Value::NativeFunction("green".to_string(), green_fn));

    let blue_fn: NativeFn = Rc::new(|args: &[Value]| {
        let text = args.first().map(|v| v.to_string()).unwrap_or_default();
        Ok(Value::String(format!("\x1b[34m{}\x1b[0m", text)))
    });
    map.insert("blue".to_string(), Value::NativeFunction("blue".to_string(), blue_fn));

    let yellow_fn: NativeFn = Rc::new(|args: &[Value]| {
        let text = args.first().map(|v| v.to_string()).unwrap_or_default();
        Ok(Value::String(format!("\x1b[33m{}\x1b[0m", text)))
    });
    map.insert("yellow".to_string(), Value::NativeFunction("yellow".to_string(), yellow_fn));

    let cyan_fn: NativeFn = Rc::new(|args: &[Value]| {
        let text = args.first().map(|v| v.to_string()).unwrap_or_default();
        Ok(Value::String(format!("\x1b[36m{}\x1b[0m", text)))
    });
    map.insert("cyan".to_string(), Value::NativeFunction("cyan".to_string(), cyan_fn));

    let magenta_fn: NativeFn = Rc::new(|args: &[Value]| {
        let text = args.first().map(|v| v.to_string()).unwrap_or_default();
        Ok(Value::String(format!("\x1b[35m{}\x1b[0m", text)))
    });
    map.insert("magenta".to_string(), Value::NativeFunction("magenta".to_string(), magenta_fn));

    let bold_fn: NativeFn = Rc::new(|args: &[Value]| {
        let text = args.first().map(|v| v.to_string()).unwrap_or_default();
        Ok(Value::String(format!("\x1b[1m{}\x1b[0m", text)))
    });
    map.insert("bold".to_string(), Value::NativeFunction("bold".to_string(), bold_fn));

    // cli.clear()
    let clear_fn: NativeFn = Rc::new(|_args: &[Value]| {
        if cfg!(windows) {
            let _ = std::process::Command::new("cmd").args(["/c", "cls"]).status();
        } else {
            print!("\x1b[2J\x1b[1;1H");
        }
        Ok(Value::Null)
    });
    map.insert("clear".to_string(), Value::NativeFunction("clear".to_string(), clear_fn));

    Value::Map(Rc::new(RefCell::new(map)))
}
