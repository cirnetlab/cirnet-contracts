use jsonschema::{Draft, ValidationOptions, Validator, error::ValidationErrorKind};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    error::Error,
    fmt, fs,
    path::{Component, Path, PathBuf},
    sync::OnceLock,
};

mod bundle;
mod design;
mod numeric;
mod profile;
mod schema_patterns;
mod semantics;

use numeric::NumberView;
pub(crate) use semantics::validate_capture_semantics;

const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";
const CANONICAL_HOST: &str = "https://cirnet.local/";
const LEGACY_HOST: &str = "https://netcircuit.local/";

const MAP_SCHEMAS: &[&str] = &["properties", "patternProperties", "dependentSchemas"];
const ARRAY_SCHEMAS: &[&str] = &["allOf", "anyOf", "oneOf", "prefixItems"];
const SINGLE_SCHEMAS: &[&str] = &[
    "items",
    "contains",
    "additionalProperties",
    "propertyNames",
    "not",
    "if",
    "then",
    "else",
    "unevaluatedProperties",
    "unevaluatedItems",
    "contentSchema",
];

#[derive(Clone, Debug)]
pub struct RegistryError(pub(crate) String);

impl fmt::Display for RegistryError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(&self.0)
    }
}

impl Error for RegistryError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ValidationIssue {
    pub code: String,
    pub path: String,
    pub severity: String,
    pub profile: String,
    pub message: String,
}

impl ValidationIssue {
    fn new(code: &str, path: &str, message: impl Into<String>, profile: &str) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
            severity: "error".into(),
            profile: profile.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug)]
pub enum ValidationError {
    UnknownContract(String),
    UnknownProfile(String),
    Invalid(ValidationIssue),
    Registry(RegistryError),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownContract(name) => write!(output, "Unknown contract {name}"),
            Self::UnknownProfile(name) => write!(output, "Unknown validation profile {name}"),
            Self::Invalid(issue) => write!(
                output,
                "{} at {}: {}",
                issue.code, issue.path, issue.message
            ),
            Self::Registry(error) => error.fmt(output),
        }
    }
}

impl Error for ValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Registry(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RegistryError> for ValidationError {
    fn from(error: RegistryError) -> Self {
        Self::Registry(error)
    }
}

impl From<ValidationIssue> for ValidationError {
    fn from(issue: ValidationIssue) -> Self {
        Self::Invalid(issue)
    }
}

#[derive(Deserialize)]
struct Manifest {
    contracts: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    kind: String,
    path: String,
    uri: String,
    pointer: String,
    visibility: String,
}

struct ProfileRuntime {
    overlay: profile::Profile,
    options: OnceLock<Result<ValidationOptions, RegistryError>>,
    validators: Vec<OnceLock<Result<NamedValidator, RegistryError>>>,
}

struct NamedValidator {
    schema: Validator,
    properties_before_required: OnceLock<Option<Validator>>,
}

/// Local Draft 2020-12 resources, named validation profiles and contract semantics.
pub struct Registry {
    entries: Vec<Entry>,
    names: HashMap<String, usize>,
    documents: HashMap<String, Value>,
    profiles: HashMap<String, ProfileRuntime>,
    root: PathBuf,
    design: OnceLock<Result<design::Catalogs, RegistryError>>,
}

impl Registry {
    /// Read the canonical manifest, schemas, profiles and design catalogs from a source root.
    pub fn load(root: &Path) -> Result<Self, RegistryError> {
        let root = &root
            .canonicalize()
            .map_err(|error| RegistryError(format!("{}: {error}", root.display())))?;
        let Manifest { contracts } = read_json(root, "contracts/manifest.json")?;
        let mut names = HashMap::new();
        let mut documents = HashMap::new();
        let mut uri_paths = HashMap::new();
        let mut identities = HashSet::new();
        let mut entries = Vec::new();
        for entry in contracts {
            if !identities.insert(entry.id.clone()) {
                return Err(RegistryError(format!(
                    "Duplicate contract identity: {}",
                    entry.id
                )));
            }
            if entry.kind != "schema" {
                continue;
            }
            if !entry.uri.starts_with(CANONICAL_HOST) || entry.uri.contains('#') {
                return Err(RegistryError(format!(
                    "Invalid canonical schema URI: {}",
                    entry.uri
                )));
            }
            match uri_paths.insert(entry.uri.clone(), entry.path.clone()) {
                Some(path) if path != entry.path => {
                    return Err(RegistryError(format!(
                        "Schema URI maps to multiple paths: {}",
                        entry.uri
                    )));
                }
                Some(_) => {}
                None => {
                    let mut value: Value = read_json(root, &entry.path)?;
                    if let Some(object) = value.as_object_mut() {
                        object.insert("$id".into(), Value::String(entry.uri.clone()));
                    }
                    documents.insert(entry.uri.clone(), value);
                }
            }
            if (!entry.pointer.is_empty() && !entry.pointer.starts_with('/'))
                || !documents[&entry.uri]
                    .pointer(&entry.pointer)
                    .is_some_and(|target| target.is_object() || target.is_boolean())
            {
                return Err(RegistryError(format!(
                    "Invalid schema pointer for {}",
                    entry.id
                )));
            }
            names.insert(entry.id.clone(), entries.len());
            entries.push(entry);
        }
        let profiles: profile::Profiles = read_json(root, "contracts/profiles.json")?;
        let profiles = profiles
            .profiles
            .into_iter()
            .map(|(name, overlay)| {
                (
                    name,
                    ProfileRuntime {
                        overlay,
                        options: OnceLock::new(),
                        validators: (0..entries.len()).map(|_| OnceLock::new()).collect(),
                    },
                )
            })
            .collect();

        Ok(Self {
            entries,
            names,
            documents,
            profiles,
            root: root.to_owned(),
            design: OnceLock::new(),
        })
    }

    /// Look up a canonical schema or JSON Pointer fragment. The historical host is an alias.
    pub fn load_schema(&self, uri: &str) -> Result<Value, RegistryError> {
        let (document, pointer) = uri.split_once('#').unwrap_or((uri, ""));
        let canonical = canonical_uri(document);
        self.documents
            .get(&canonical)
            .and_then(|schema| schema.pointer(pointer))
            .cloned()
            .ok_or_else(|| RegistryError(format!("Unresolved offline schema: {uri}")))
    }

    /// Return a detached valid instance using the legacy reader's numeric model.
    /// Preserve lexical integers exactly and leave optional properties absent.
    pub fn validate(
        &self,
        name: &str,
        profile: &str,
        value: &Value,
    ) -> Result<Value, ValidationError> {
        match self.validate_at(name, profile, value, "")? {
            Cow::Borrowed(value) => Ok(value.clone()),
            Cow::Owned(_) => {
                Ok(normalized_value(value, "", profile, NumberView::Output)?.into_owned())
            }
        }
    }

    fn validate_at<'a>(
        &self,
        name: &str,
        profile: &str,
        value: &'a Value,
        path: &str,
    ) -> Result<Cow<'a, Value>, ValidationError> {
        let index = *self
            .names
            .get(name)
            .ok_or_else(|| ValidationError::UnknownContract(name.into()))?;
        let runtime = self
            .profiles
            .get(profile)
            .ok_or_else(|| ValidationError::UnknownProfile(profile.into()))?;
        let schema_value = normalized_value(value, path, profile, NumberView::Schema)?;
        let validator = runtime.validators[index]
            .get_or_init(|| self.compile(index, runtime))
            .as_ref()
            .map_err(Clone::clone)?;
        if let Err(mut error) = validator.schema.validate(schema_value.as_ref()) {
            if error.instance_path().as_str().is_empty()
                && matches!(error.schema_path().as_str(), "/$ref/required" | "/required")
                && matches!(error.kind(), ValidationErrorKind::Required { .. })
                && let Some(preceding) = validator
                    .properties_before_required
                    .get_or_init(|| self.compile_properties_before_required(index, runtime))
                && let Err(earlier) = preceding.validate(schema_value.as_ref())
            {
                error = earlier;
            }
            let mut location = format!("{path}{}", error.instance_path());
            let schema_path = error.schema_path().as_str();
            let keyword = match error.kind() {
                ValidationErrorKind::Required { property } => {
                    if let Some(property) = property.as_str() {
                        location = child(&location, property);
                    }
                    "required"
                }
                ValidationErrorKind::FalseSchema => "validation",
                _ => schema_path
                    .rsplit('/')
                    .next()
                    .filter(|last| !last.is_empty())
                    .unwrap_or("validation"),
            };
            let code = format!("SCHEMA_{}", keyword.to_ascii_uppercase());
            return Err(ValidationIssue::new(&code, &location, error.to_string(), profile).into());
        }
        self.semantics(name, profile, value, path)?;
        Ok(schema_value)
    }

    fn catalogs(&self) -> Result<&design::Catalogs, RegistryError> {
        self.design
            .get_or_init(|| design::Catalogs::load(&self.root))
            .as_ref()
            .map_err(Clone::clone)
    }

    fn compile(
        &self,
        index: usize,
        runtime: &ProfileRuntime,
    ) -> Result<NamedValidator, RegistryError> {
        let options = runtime
            .options
            .get_or_init(|| {
                let mut resources = Vec::with_capacity(self.documents.len() * 2);
                for (uri, document) in &self.documents {
                    let schema = runtime.overlay.apply("", document);
                    let mut schema = numeric::normalize_binary64(&schema, "", NumberView::Schema)
                        .map_err(|location| {
                            RegistryError(format!("Nonfinite schema number at {uri}#{location}"))
                        })?
                        .into_owned();
                    schema_patterns::adapt(&mut schema);
                    let alias = uri.replacen(CANONICAL_HOST, LEGACY_HOST, 1);
                    resources.extend([
                        (
                            uri.clone(),
                            Draft::Draft202012.create_resource(schema.clone()),
                        ),
                        (alias, Draft::Draft202012.create_resource(schema)),
                    ]);
                }
                let registry = jsonschema::Registry::options()
                    .draft(Draft::Draft202012)
                    .build(resources)
                    .map_err(|error| RegistryError(error.to_string()))?;
                Ok(jsonschema::draft202012::options().with_registry(registry))
            })
            .as_ref()
            .map_err(Clone::clone)?;
        let entry = &self.entries[index];
        let mut uri = entry.uri.clone();
        if !entry.pointer.is_empty() {
            uri.push('#');
            uri.push_str(&entry.pointer);
        }
        options
            .build(&json!({"$ref": uri}))
            .map(|schema| NamedValidator {
                schema,
                properties_before_required: OnceLock::new(),
            })
            .map_err(|error| RegistryError(format!("Cannot compile {}: {error}", entry.id)))
    }

    // jsonschema combines properties into its additionalProperties check.
    // If that postpones properties past required, recover the first issue
    // by replaying only the declared properties. Validation still short-circuits.
    fn compile_properties_before_required(
        &self,
        index: usize,
        runtime: &ProfileRuntime,
    ) -> Option<Validator> {
        let entry = &self.entries[index];
        let schema = self.documents[&entry.uri]
            .pointer(&entry.pointer)?
            .as_object()?;
        let position = |key| schema.keys().position(|item| item == key);
        let required = position("required")?;
        if !(position("properties")? < required && required < position("additionalProperties")?)
            || !matches!(
                schema.get("additionalProperties"),
                Some(Value::Bool(false) | Value::Object(_))
            )
        {
            return None;
        }

        let mut uri = url::Url::parse(&entry.uri).ok()?;
        let prefix = child(&entry.pointer, "properties");
        let properties: Map<_, _> = schema
            .get("properties")?
            .as_object()?
            .keys()
            .map(|key| {
                uri.set_fragment(Some(&child(&prefix, key)));
                (key.clone(), json!({"$ref":uri.as_str()}))
            })
            .collect();
        runtime
            .options
            .get()?
            .as_ref()
            .ok()?
            .build(&json!({"properties":properties}))
            .ok()
    }
}

fn canonical_uri(uri: &str) -> String {
    uri.replacen(LEGACY_HOST, CANONICAL_HOST, 1)
}

fn child(path: &str, key: impl fmt::Display) -> String {
    format!(
        "{path}/{}",
        key.to_string().replace('~', "~0").replace('/', "~1")
    )
}

fn normalized_value<'a>(
    value: &'a Value,
    path: &str,
    profile: &str,
    view: NumberView,
) -> Result<Cow<'a, Value>, ValidationIssue> {
    numeric::normalize_binary64(value, path, view).map_err(|location| {
        ValidationIssue::new(
            "NONFINITE_NUMBER",
            &location,
            "Nonfinite numeric value",
            profile,
        )
    })
}

fn read_json<T: serde::de::DeserializeOwned>(
    root: &Path,
    relative: &str,
) -> Result<T, RegistryError> {
    let relative = Path::new(relative);
    if !relative
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(RegistryError(format!(
            "Invalid local resource path: {}",
            relative.display()
        )));
    }
    let path = root.join(relative);
    let path = path
        .canonicalize()
        .map_err(|error| RegistryError(format!("{}: {error}", path.display())))?;
    if !path.starts_with(root) {
        return Err(RegistryError(format!(
            "Local resource {} resolves outside the source root",
            relative.display(),
        )));
    }
    let bytes =
        fs::read(&path).map_err(|error| RegistryError(format!("{}: {error}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| RegistryError(format!("{}: {error}", path.display())))
}
