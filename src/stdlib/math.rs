use crate::core::value::{NativeFn, Value};
use indexmap::IndexMap;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_math_module() -> Value {
    let mut map = IndexMap::new();

    // Constants
    map.insert("PI".to_string(), Value::Float(std::f64::consts::PI));
    map.insert("E".to_string(), Value::Float(std::f64::consts::E));

    // random(min, max)
    let random_fn: NativeFn = Rc::new(|args: &[Value]| {
        let mut rng = rand::thread_rng();
        if args.len() >= 2 {
            let min_val = match &args[0] {
                Value::Int(i) => *i,
                Value::Float(f) => *f as i64,
                _ => 0,
            };
            let max_val = match &args[1] {
                Value::Int(i) => *i,
                Value::Float(f) => *f as i64,
                _ => 100,
            };
            if min_val > max_val {
                return Ok(Value::Int(rng.gen_range(max_val..=min_val)));
            }
            Ok(Value::Int(rng.gen_range(min_val..=max_val)))
        } else if args.len() == 1 {
            let max_val = match &args[0] {
                Value::Int(i) => *i,
                Value::Float(f) => *f as i64,
                _ => 100,
            };
            Ok(Value::Int(rng.gen_range(0..=max_val)))
        } else {
            Ok(Value::Float(rng.gen::<f64>()))
        }
    });
    map.insert("random".to_string(), Value::NativeFunction("random".to_string(), random_fn));

    // sqrt
    let sqrt_fn: NativeFn = Rc::new(|args: &[Value]| {
        let n = args.first().ok_or("Expected number")?;
        let val = match n {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => return Err("Expected number for sqrt".to_string()),
        };
        Ok(Value::Float(val.sqrt()))
    });
    map.insert("sqrt".to_string(), Value::NativeFunction("sqrt".to_string(), sqrt_fn));

    // abs
    let abs_fn: NativeFn = Rc::new(|args: &[Value]| {
        let n = args.first().ok_or("Expected number")?;
        match n {
            Value::Int(i) => Ok(Value::Int(i.abs())),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            _ => Err("Expected number for abs".to_string()),
        }
    });
    map.insert("abs".to_string(), Value::NativeFunction("abs".to_string(), abs_fn));

    // sin, cos, tan
    let sin_fn: NativeFn = Rc::new(|args: &[Value]| {
        let val = match args.first().ok_or("Expected number")? {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => return Err("Expected number".to_string()),
        };
        Ok(Value::Float(val.sin()))
    });
    map.insert("sin".to_string(), Value::NativeFunction("sin".to_string(), sin_fn));

    let cos_fn: NativeFn = Rc::new(|args: &[Value]| {
        let val = match args.first().ok_or("Expected number")? {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => return Err("Expected number".to_string()),
        };
        Ok(Value::Float(val.cos()))
    });
    map.insert("cos".to_string(), Value::NativeFunction("cos".to_string(), cos_fn));

    let tan_fn: NativeFn = Rc::new(|args: &[Value]| {
        let val = match args.first().ok_or("Expected number")? {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => return Err("Expected number".to_string()),
        };
        Ok(Value::Float(val.tan()))
    });
    map.insert("tan".to_string(), Value::NativeFunction("tan".to_string(), tan_fn));

    // floor, ceil, round
    let floor_fn: NativeFn = Rc::new(|args: &[Value]| {
        let val = match args.first().ok_or("Expected number")? {
            Value::Int(i) => *i,
            Value::Float(f) => f.floor() as i64,
            _ => return Err("Expected number".to_string()),
        };
        Ok(Value::Int(val))
    });
    map.insert("floor".to_string(), Value::NativeFunction("floor".to_string(), floor_fn));

    let ceil_fn: NativeFn = Rc::new(|args: &[Value]| {
        let val = match args.first().ok_or("Expected number")? {
            Value::Int(i) => *i,
            Value::Float(f) => f.ceil() as i64,
            _ => return Err("Expected number".to_string()),
        };
        Ok(Value::Int(val))
    });
    map.insert("ceil".to_string(), Value::NativeFunction("ceil".to_string(), ceil_fn));

    let round_fn: NativeFn = Rc::new(|args: &[Value]| {
        let val = match args.first().ok_or("Expected number")? {
            Value::Int(i) => *i,
            Value::Float(f) => f.round() as i64,
            _ => return Err("Expected number".to_string()),
        };
        Ok(Value::Int(val))
    });
    map.insert("round".to_string(), Value::NativeFunction("round".to_string(), round_fn));

    // pow
    let pow_fn: NativeFn = Rc::new(|args: &[Value]| {
        if args.len() < 2 {
            return Err("Expected 2 arguments for pow(base, exponent)".to_string());
        }
        let base = match &args[0] {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => return Err("Expected number base".to_string()),
        };
        let exp = match &args[1] {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => return Err("Expected number exponent".to_string()),
        };
        Ok(Value::Float(base.powf(exp)))
    });
    map.insert("pow".to_string(), Value::NativeFunction("pow".to_string(), pow_fn));

    Value::Map(Rc::new(RefCell::new(map)))
}
