use super::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_child(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.values.get(name) {
            return Some(val.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return;
        }

        if let Some(parent) = &self.parent {
            if parent.borrow().get(name).is_some() {
                parent.borrow_mut().assign(name, value);
                return;
            }
        }

        self.values.insert(name.to_string(), value);
    }

    pub fn keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.values.keys().cloned().collect();
        if let Some(parent) = &self.parent {
            keys.extend(parent.borrow().keys());
        }
        keys.sort();
        keys.dedup();
        keys
    }
}
