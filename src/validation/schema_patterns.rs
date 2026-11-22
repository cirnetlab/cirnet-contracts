use super::{ARRAY_SCHEMAS, MAP_SCHEMAS, SINGLE_SCHEMAS};
use serde_json::Value;

/// Adapt only runtime copies and only schema positions. Defaults, examples,
/// SDK bundles and strict end-of-input patterns keep their original text.
pub(super) fn adapt(schema: &mut Value) {
    let Some(schema) = schema.as_object_mut() else {
        return;
    };
    if let Some(Value::String(pattern)) = schema.get_mut("pattern") {
        adapt_pattern(pattern);
    }
    for key in std::iter::once("$defs").chain(MAP_SCHEMAS.iter().copied()) {
        if let Some(Value::Object(children)) = schema.get_mut(key) {
            for child in children.values_mut() {
                adapt(child);
            }
            if key == "patternProperties" {
                *children = std::mem::take(children)
                    .into_iter()
                    .map(|(mut pattern, child)| {
                        adapt_pattern(&mut pattern);
                        (pattern, child)
                    })
                    .collect();
            }
        }
    }
    for key in ARRAY_SCHEMAS {
        if let Some(Value::Array(children)) = schema.get_mut(*key) {
            children.iter_mut().for_each(adapt);
        }
    }
    for key in SINGLE_SCHEMAS {
        if let Some(child) = schema.get_mut(*key) {
            adapt(child);
        }
    }
}

fn adapt_pattern(pattern: &mut String) {
    if let Some(prefix) = pattern.strip_suffix('$') {
        let escapes = prefix
            .bytes()
            .rev()
            .take_while(|byte| *byte == b'\\')
            .count();
        if escapes.is_multiple_of(2) {
            // Python's $ also matches immediately before the final LF.
            pattern.pop();
            pattern.push_str(r"(?=\n?$)");
        }
    }
}
