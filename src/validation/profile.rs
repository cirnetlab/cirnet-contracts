use super::numeric;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{cmp::Ordering, collections::HashMap};

#[derive(Deserialize)]
pub(super) struct Profiles {
    pub(super) profiles: HashMap<String, Profile>,
}

#[derive(Deserialize)]
pub(super) struct Profile {
    integer: Option<Map<String, Value>>,
    definitions: Map<String, Value>,
}

impl Profile {
    pub(super) fn apply(&self, name: &str, schema: &Value) -> Value {
        let mut result = schema.clone();
        if let Some(integer) = &self.integer {
            bound(&mut result, integer);
        }
        if let Some(definitions) = result.get_mut("$defs").and_then(Value::as_object_mut) {
            for (name, overlay) in &self.definitions {
                if let Some(target) = definitions.get_mut(name) {
                    intersect_values(target, overlay);
                }
            }
        } else if result.get("$ref").is_none() {
            if let Some(overlay) = self.definitions.get(name) {
                intersect_values(&mut result, overlay);
            }
        }
        result
    }
}

fn bound(value: &mut Value, integer: &Map<String, Value>) {
    let Some(rule) = value.as_object_mut() else {
        return;
    };
    if rule.get("type").and_then(Value::as_str) == Some("integer") {
        intersect(rule, integer);
    }
    for item in rule.values_mut() {
        if let Value::Array(items) = item {
            for item in items {
                bound(item, integer);
            }
        } else {
            bound(item, integer);
        }
    }
}

fn intersect_values(target: &mut Value, overlay: &Value) {
    if let (Some(target), Some(overlay)) = (target.as_object_mut(), overlay.as_object()) {
        intersect(target, overlay);
    }
}

fn intersect(target: &mut Map<String, Value>, overlay: &Map<String, Value>) {
    for (key, value) in overlay {
        if let Some(current) = target.get_mut(key) {
            if current.is_object() && value.is_object() {
                intersect_values(current, value);
                continue;
            }
            let order = numeric::compare(value, current);
            if key == "maximum" && order == Ordering::Greater
                || key == "minimum" && order == Ordering::Less
            {
                continue;
            }
            *current = value.clone();
        } else {
            target.insert(key.clone(), value.clone());
        }
    }
}
