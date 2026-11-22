use super::{
    ARRAY_SCHEMAS, DIALECT, MAP_SCHEMAS, Registry, RegistryError, SINGLE_SCHEMAS, canonical_uri,
};
use serde_json::{Map, Value, json};
use url::Url;

impl Registry {
    /// Relocate manifest-identified references into a single generator input.
    ///
    /// Map insertion order follows the canonical JSON and manifest. A public
    /// bundle may only refer to other public identities.
    pub fn schema_bundle(
        &self,
        public_only: bool,
        profile: &str,
        ref_prefix: &str,
    ) -> Result<Value, RegistryError> {
        let runtime = self
            .profiles
            .get(profile)
            .ok_or_else(|| RegistryError(format!("Unknown validation profile {profile}")))?;
        let mut definitions = Map::new();
        let mut properties = Map::new();
        let mut required = Vec::new();
        for entry in self
            .entries
            .iter()
            .filter(|entry| !public_only || entry.visibility == "public")
        {
            let mut schema = self.documents[&entry.uri]
                .pointer(&entry.pointer)
                .expect("checked manifest pointer")
                .clone();
            if let Some(object) = schema.as_object_mut() {
                object.shift_remove("title");
            }
            let base = Url::parse(&entry.uri).map_err(|error| RegistryError(error.to_string()))?;
            let mut schema = runtime.overlay.apply(&entry.id, &schema);
            self.relocate(&mut schema, &base, public_only, ref_prefix)?;
            definitions.insert(entry.id.clone(), schema);
            properties.insert(
                entry.id.clone(),
                json!({"$ref": format!("{ref_prefix}{}", entry.id)}),
            );
            required.push(entry.id.clone());
        }

        Ok(json!({
            "$schema": DIALECT,
            "$id": format!("https://cirnet.local/contracts/generated/{profile}"),
            "$defs": definitions,
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false,
        }))
    }

    fn relocate(
        &self,
        schema: &mut Value,
        base: &Url,
        public_only: bool,
        ref_prefix: &str,
    ) -> Result<(), RegistryError> {
        let Some(object) = schema.as_object_mut() else {
            return Ok(());
        };
        let base = base
            .join(object.get("$id").and_then(Value::as_str).unwrap_or(""))
            .map_err(|error| RegistryError(error.to_string()))?;
        for name in ["$id", "$schema", "$defs"] {
            object.shift_remove(name);
        }
        if object.contains_key("$dynamicRef") {
            return Err(RegistryError("Code generation does not support $dynamicRef relocation; runtime validation supports it".into()));
        }
        if let Some(reference) = object.get_mut("$ref") {
            let value = reference
                .as_str()
                .ok_or_else(|| RegistryError("Schema $ref must be a string".into()))?;
            let uri = base
                .join(value)
                .map_err(|error| RegistryError(error.to_string()))?;
            *reference =
                Value::String(self.reference_target(uri.as_str(), public_only, ref_prefix)?);
        }
        for key in MAP_SCHEMAS {
            if let Some(entries) = object.get_mut(*key).and_then(Value::as_object_mut) {
                for child in entries.values_mut() {
                    self.relocate(child, &base, public_only, ref_prefix)?;
                }
            }
        }
        for key in ARRAY_SCHEMAS {
            if let Some(entries) = object.get_mut(*key).and_then(Value::as_array_mut) {
                for child in entries {
                    self.relocate(child, &base, public_only, ref_prefix)?;
                }
            }
        }
        for key in SINGLE_SCHEMAS {
            if let Some(child) = object.get_mut(*key) {
                self.relocate(child, &base, public_only, ref_prefix)?;
            }
        }
        Ok(())
    }

    fn reference_target(
        &self,
        uri: &str,
        public_only: bool,
        ref_prefix: &str,
    ) -> Result<String, RegistryError> {
        let (document, pointer) = uri.split_once('#').unwrap_or((uri, ""));
        let document = canonical_uri(document);
        let unresolved = || {
            RegistryError(format!(
                "Unmanifested or private generator reference: {uri}"
            ))
        };
        let entry = self
            .entries
            .iter()
            .filter(|entry| {
                entry.uri == document
                    && pointer
                        .strip_prefix(&entry.pointer)
                        .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('/'))
            })
            .max_by_key(|entry| entry.pointer.len())
            .ok_or_else(unresolved)?;
        if public_only && entry.visibility != "public" {
            return Err(unresolved());
        }
        let suffix = &pointer[entry.pointer.len()..];
        if suffix.contains("/$defs/") {
            return Err(RegistryError(format!(
                "Unmanifested nested definition in generator reference: {uri}"
            )));
        }
        let value = self.documents[&entry.uri]
            .pointer(pointer)
            .ok_or_else(|| RegistryError(format!("Unresolved generator reference: {uri}")))?;
        if !value.is_object() && !value.is_boolean() {
            return Err(RegistryError(format!(
                "Generator reference must address a schema: {uri}"
            )));
        }
        Ok(format!("{ref_prefix}{}{suffix}", entry.id))
    }
}
