//! Parse JSON inputs without treating an object as serde_json's private number tag.

use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde_json::{Map, Value, value::RawValue};
use std::fmt;

pub(crate) struct RawObject<'a>(pub(crate) Vec<(String, &'a RawValue)>);

impl<'de> Deserialize<'de> for RawObject<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor;

        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = RawObject<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut members = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    members.push(entry);
                }
                Ok(RawObject(members))
            }
        }

        deserializer.deserialize_map(ObjectVisitor)
    }
}

/// Preserve JSON object and number types, including integers larger than u64.
pub fn parse(bytes: &[u8]) -> serde_json::Result<Value> {
    let raw: &RawValue = serde_json::from_slice(bytes)?;
    let mut pending = vec![Pending::Read(raw, 0)];
    let mut values = Vec::new();
    // The top of the value stack holds the completed children of each container.
    while let Some(item) = pending.pop() {
        match item {
            Pending::Read(raw, depth) => {
                if depth > 1000 {
                    return Err(de::Error::custom("JSON nesting limit exceeded"));
                }
                let text = raw.get();
                match text.as_bytes()[0] {
                    b'{' => {
                        let RawObject(members) = serde_json::from_str(text)?;
                        let (keys, children): (Vec<_>, Vec<_>) = members.into_iter().unzip();
                        pending.push(Pending::Object(keys));
                        pending.extend(
                            children
                                .into_iter()
                                .rev()
                                .map(|child| Pending::Read(child, depth + 1)),
                        );
                    }
                    b'[' => {
                        let children: Vec<&RawValue> = serde_json::from_str(text)?;
                        pending.push(Pending::Array(children.len()));
                        pending.extend(
                            children
                                .into_iter()
                                .rev()
                                .map(|child| Pending::Read(child, depth + 1)),
                        );
                    }
                    _ => values.push(serde_json::from_str(text)?),
                }
            }
            Pending::Array(count) => {
                let children = values.split_off(values.len() - count);
                values.push(Value::Array(children));
            }
            Pending::Object(keys) => {
                let children = values.split_off(values.len() - keys.len());
                let members: Map<_, _> = keys.into_iter().zip(children).collect();
                values.push(Value::Object(members));
            }
        }
    }
    Ok(values.pop().expect("a validated JSON root"))
}

enum Pending<'a> {
    Read(&'a RawValue, usize),
    Array(usize),
    Object(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parsing_preserves_wire_types_and_last_duplicate_value() {
        let input = br#"{
            "exact": 18446744073709551616,
            "array": [{"\u0024serde_json::private::Number": "1"}],
            "repeated": 0,
            "repeated": 2
        }"#;
        let parsed = parse(input).unwrap();
        assert_eq!(parsed["exact"].to_string(), "18446744073709551616");
        assert_eq!(parsed["array"][0]["$serde_json::private::Number"], "1");
        assert_eq!(parsed["repeated"], 2);
    }

    #[test]
    fn parses_nested_legacy_properties_and_rejects_excessive_depth() {
        let legacy = format!("{}0{}", "[".repeat(300), "]".repeat(300));
        let parsed = parse(legacy.as_bytes()).unwrap();
        let mut inner = &parsed;
        for _ in 0..300 {
            inner = &inner[0];
        }
        assert_eq!(*inner, 0);

        let excessive = format!("{}0{}", "[".repeat(1001), "]".repeat(1001));
        assert!(parse(excessive.as_bytes()).is_err());
    }
}
