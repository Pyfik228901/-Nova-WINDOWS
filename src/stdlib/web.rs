use crate::core::value::{NativeFn, Value};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct WebRouter {
    pub routes: HashMap<(String, String), Value>, // (METHOD, PATH) -> Handler Function
}

impl WebRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }
}

pub fn create_web_module(router_ref: Rc<RefCell<WebRouter>>) -> Value {
    let mut map = IndexMap::new();

    // web.get(path, handler)
    let router_clone = router_ref.clone();
    let get_fn: NativeFn = Rc::new(move |args: &[Value]| {
        let path = args.first().ok_or("Expected route path")?.to_string();
        let handler = args.get(1).ok_or("Expected route handler function")?.clone();
        router_clone.borrow_mut().routes.insert(("GET".to_string(), path), handler);
        Ok(Value::Null)
    });
    map.insert("get".to_string(), Value::NativeFunction("get".to_string(), get_fn));

    // web.post(path, handler)
    let router_clone2 = router_ref.clone();
    let post_fn: NativeFn = Rc::new(move |args: &[Value]| {
        let path = args.first().ok_or("Expected route path")?.to_string();
        let handler = args.get(1).ok_or("Expected route handler function")?.clone();
        router_clone2.borrow_mut().routes.insert(("POST".to_string(), path), handler);
        Ok(Value::Null)
    });
    map.insert("post".to_string(), Value::NativeFunction("post".to_string(), post_fn));

    // web.json_parse(str)
    let json_parse_fn: NativeFn = Rc::new(|args: &[Value]| {
        let s = args.first().ok_or("Expected JSON string")?.to_string();
        let val: serde_json::Value = serde_json::from_str(&s)
            .map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(crate::ffi::grassgreen::GrassGreenBridge::json_to_value(&val))
    });
    map.insert("json_parse".to_string(), Value::NativeFunction("json_parse".to_string(), json_parse_fn));

    // web.json_stringify(val)
    let json_stringify_fn: NativeFn = Rc::new(|args: &[Value]| {
        let val = args.first().ok_or("Expected value to stringify")?;
        let json = value_to_json(val);
        let s = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("JSON stringify error: {}", e))?;
        Ok(Value::String(s))
    });
    map.insert("json_stringify".to_string(), Value::NativeFunction("json_stringify".to_string(), json_stringify_fn));

    // web.html(title, body)
    let html_fn: NativeFn = Rc::new(|args: &[Value]| {
        let title = args.first().map(|v| v.to_string()).unwrap_or_else(|| "N&S Web Application".to_string());
        let body = args.get(1).map(|v| v.to_string()).unwrap_or_default();

        let doc = format!(r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        :root {{
            --bg-color: #11111b;
            --surface: #1e1e2e;
            --text-color: #cdd6f4;
            --accent: #89b4fa;
            --accent-green: #a6e3a1;
            --border: #313244;
        }}
        body {{
            background: var(--bg-color);
            color: var(--text-color);
            font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
            margin: 0;
            padding: 2rem;
            display: flex;
            flex-direction: column;
            align-items: center;
            min-height: 100vh;
            box-sizing: border-box;
        }}
        .container {{
            width: 100%;
            max-width: 800px;
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 2rem;
            box-shadow: 0 8px 24px rgba(0,0,0,0.4);
        }}
        h1, h2, h3 {{ color: var(--accent); margin-top: 0; }}
        a {{ color: var(--accent); text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        button, .btn {{
            background: var(--accent);
            color: var(--bg-color);
            border: none;
            padding: 10px 20px;
            border-radius: 6px;
            font-weight: 600;
            cursor: pointer;
            transition: opacity 0.2s;
        }}
        button:hover, .btn:hover {{ opacity: 0.85; }}
    </style>
</head>
<body>
    <div class="container">
        {}
    </div>
</body>
</html>"#, title, body);

        Ok(Value::String(doc))
    });
    map.insert("html".to_string(), Value::NativeFunction("html".to_string(), html_fn));

    // web.tag(name, attrs, content)
    let tag_fn: NativeFn = Rc::new(|args: &[Value]| {
        let name = args.first().ok_or("Expected tag name")?.to_string();
        let attrs = args.get(1);
        let content = args.get(2).map(|v| v.to_string()).unwrap_or_default();

        let mut attr_str = String::new();
        if let Some(Value::Map(m)) = attrs {
            for (k, v) in m.borrow().iter() {
                attr_str.push_str(&format!(" {}=\"{}\"", k, v));
            }
        }

        Ok(Value::String(format!("<{}{}>{}</{}>", name, attr_str, content, name)))
    });
    map.insert("tag".to_string(), Value::NativeFunction("tag".to_string(), tag_fn));

    Value::Map(Rc::new(RefCell::new(map)))
}

pub fn value_to_json(val: &Value) -> serde_json::Value {
    match val {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Array(a) => {
            let elements: Vec<serde_json::Value> = a.borrow().iter().map(value_to_json).collect();
            serde_json::Value::Array(elements)
        }
        Value::Map(m) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in m.borrow().iter() {
                obj.insert(k.clone(), value_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
        Value::Function(f) => {
            serde_json::Value::String(format!("<function {}>", f.name.as_deref().unwrap_or("anon")))
        }
        Value::NativeFunction(name, _) => {
            serde_json::Value::String(format!("<native_function {}>", name))
        }
    }
}
