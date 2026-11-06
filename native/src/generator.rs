mod catalog;
mod operations;

use crate::validation::Registry;
use serde::Deserialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Artifacts = BTreeMap<String, Vec<u8>>;

const PROFILES: [&str; 3] = ["structural", "editor", "transport"];
const BROWSER: &str = "contracts/browser";
const OPENAPI: &str = "contracts/api/openapi.json";
const MANIFEST: &str = "contracts/manifest.json";
const PROFILE_SOURCE: &str = "contracts/profiles.json";
const PROJECTED_CATALOGS: [&str; 2] = [
    "contracts/design-models/physical-parts.v1.json",
    "contracts/design-models/signal-port-catalog.json",
];

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
    visibility: String,
    version: String,
    producer: String,
    consumers: Vec<String>,
    compatibility_policy: String,
    #[serde(rename = "pointer")]
    _pointer: String,
    #[serde(rename = "validator_profile")]
    _validator_profile: String,
    #[serde(rename = "generated_artifacts")]
    _generated_artifacts: Vec<String>,
}

impl Entry {
    fn is_public_schema(&self) -> bool {
        self.kind == "schema" && self.visibility == "public"
    }
}

/// Build a new candidate tree from canonical source and the installed Node tools.
/// The destination must be absent, with an existing parent outside the source tree.
/// The complete candidate becomes visible only after the Node generators succeed.
pub fn generate(
    root: &Path,
    output_root: &Path,
    node_tools: &Path,
    openapi: Option<&Path>,
) -> Result<()> {
    let output = new_external_output(root, output_root)?;
    let source = Source(fs::canonicalize(root)?);
    let emitter = source.file("contracts/codegen/generate_typescript_contracts.mjs")?;
    let (artifacts, bundles) = project(&source, openapi)?;
    let parent = output.parent().expect("validated output parent");
    let stage = tempfile::Builder::new()
        .prefix(".cirnet-sdk-")
        .tempdir_in(parent)?;
    for (name, contents) in artifacts {
        let target = stage.path().join(name);
        fs::create_dir_all(target.parent().expect("artifact parent"))?;
        fs::write(target, contents)?;
    }

    let bundle_file = stage.path().join("codegen-input.json");
    fs::write(&bundle_file, serde_json::to_vec(&bundles)?)?;
    let status = Command::new("node")
        .arg(emitter)
        .arg(&bundle_file)
        .arg(stage.path().join(BROWSER).join("generated"))
        .arg(std::path::absolute(node_tools)?)
        .status()?;
    if !status.success() {
        return Err(format!("TypeScript/Ajv code generation failed ({status})").into());
    }
    fs::remove_file(&bundle_file)?;
    require_absent(&output)?;
    fs::rename(stage.path(), output)?;
    Ok(())
}

fn new_external_output(root: &Path, output: &Path) -> Result<PathBuf> {
    let root = fs::canonicalize(root)?;
    let name = output
        .file_name()
        .ok_or("Generation requires a fresh external output directory")?;
    let parent = output.parent().filter(|p| !p.as_os_str().is_empty());
    let parent = fs::canonicalize(parent.unwrap_or(Path::new(".")))?;
    let output = parent.join(name);
    if output.starts_with(&root) {
        return Err("Generation requires a fresh external output directory".into());
    }
    require_absent(&output)?;
    Ok(output)
}

fn require_absent(output: &Path) -> Result<()> {
    match fs::symlink_metadata(output) {
        Ok(_) => Err(format!(
            "Generation requires a fresh external output directory: {}",
            output.display()
        )
        .into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

struct Source(PathBuf);

impl Source {
    fn path(&self, relative: impl AsRef<Path>) -> Result<PathBuf> {
        let relative = relative.as_ref();
        if !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        {
            return Err(format!("Invalid canonical source path: {}", relative.display()).into());
        }
        self.contained(&self.0.join(relative))
    }

    fn contained(&self, path: &Path) -> Result<PathBuf> {
        let resolved =
            fs::canonicalize(path).map_err(|error| format!("{}: {error}", path.display()))?;
        if !resolved.starts_with(&self.0) {
            return Err(format!("{} escapes canonical source root", path.display()).into());
        }
        Ok(resolved)
    }

    fn file(&self, relative: impl AsRef<Path>) -> Result<PathBuf> {
        let path = self.path(relative)?;
        if !path.is_file() {
            return Err(format!("Expected canonical source file: {}", path.display()).into());
        }
        Ok(path)
    }

    fn read(&self, relative: impl AsRef<Path>) -> Result<Vec<u8>> {
        read(&self.file(relative)?)
    }

    fn json(&self, relative: impl AsRef<Path>) -> Result<Value> {
        read_json(&self.file(relative)?)
    }
}

fn check_manifest(source: &Source, entries: &[Entry]) -> Result<()> {
    let mut schemas = BTreeSet::new();
    for entry in entries {
        if !matches!(
            entry.kind.as_str(),
            "schema" | "protobuf" | "binary" | "data"
        ) {
            return Err(format!("Unknown contract kind: {}", entry.kind).into());
        }
        if !matches!(entry.visibility.as_str(), "public" | "internal") {
            return Err(format!("Unknown contract visibility: {}", entry.visibility).into());
        }
        if entry.producer.trim().is_empty()
            || entry.consumers.is_empty()
            || entry
                .consumers
                .iter()
                .any(|consumer| consumer.trim().is_empty())
            || entry.compatibility_policy.trim().is_empty()
            || entry.version.trim().is_empty()
        {
            return Err(
                format!("Incomplete contract ownership/version policy: {}", entry.id).into(),
            );
        }
        if entry.kind != "data" || !PROJECTED_CATALOGS.contains(&entry.path.as_str()) {
            source.file(&entry.path)?;
        }
        if entry.kind == "schema" {
            schemas.insert(Path::new(&entry.path));
        }
    }

    const NONCANONICAL: &[&str] = &[
        "compatibility",
        "fixtures",
        "generated",
        "boundary",
        "browser",
        "catalog-source",
    ];
    let folder = Path::new("contracts");
    for entry in fs::read_dir(source.path(folder)?)? {
        let relative = folder.join(entry?.file_name());
        if !relative
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| NONCANONICAL.contains(&name))
        {
            check_schema_coverage(source, &relative, &schemas)?;
        }
    }
    Ok(())
}

fn check_schema_coverage(
    source: &Source,
    relative: &Path,
    schemas: &BTreeSet<&Path>,
) -> Result<()> {
    let entry = fs::symlink_metadata(source.0.join(relative))?;
    if entry.is_dir() && !entry.file_type().is_symlink() {
        for child in fs::read_dir(source.path(relative)?)? {
            check_schema_coverage(source, &relative.join(child?.file_name()), schemas)?;
        }
    } else if relative
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".schema.json"))
        && !schemas.contains(relative)
    {
        return Err(format!("Unmanifested schema: {}", relative.display()).into());
    }
    Ok(())
}

fn project(source: &Source, openapi: Option<&Path>) -> Result<(Artifacts, Value)> {
    let manifest: Manifest = serde_json::from_slice(&source.read(MANIFEST)?)?;
    check_manifest(source, &manifest.contracts)?;
    let registry = Registry::load(&source.0)?;
    let mut artifacts = Artifacts::new();
    let mut bundles = Map::new();
    for profile in PROFILES {
        let bundle = registry.schema_bundle(true, profile, "#/$defs/")?;
        add_json(
            &mut artifacts,
            format!("{BROWSER}/schemas/{profile}.json"),
            &bundle,
            true,
        )?;
        bundles.insert(profile.to_owned(), bundle);
    }
    let public_names = object(&bundles["structural"]["$defs"], "public schema definitions")?;
    let mut profiles = source.json(PROFILE_SOURCE)?;
    let profile_definitions = profiles["profiles"]
        .as_object_mut()
        .ok_or("Expected profiles object")?;
    for profile in profile_definitions.values_mut() {
        profile["definitions"]
            .as_object_mut()
            .ok_or("Expected profile definitions")?
            .retain(|name, _| public_names.contains_key(name));
    }
    add_json(
        &mut artifacts,
        format!("{BROWSER}/schemas/profiles.json"),
        &profiles,
        true,
    )?;

    let document = match openapi {
        Some(path) => read_json(path)?,
        None => source.json(OPENAPI)?,
    };
    artifacts.insert(
        format!("{BROWSER}/generated/operations.ts"),
        operations::bindings(&document)?.into_bytes(),
    );
    let mut sorted_document = document;
    sorted_document.sort_all_objects();
    add_json(&mut artifacts, OPENAPI, &sorted_document, true)?;

    let layout = source.json("contracts/capture/layout.json")?;
    registry.validate("NCAPLayout", "structural", &layout)?;
    artifacts.insert(
        format!("{BROWSER}/generated/captureLayout.ts"),
        format!(
            "// Generated by scripts/generate_contracts.py from contracts/capture/layout.json.\nexport const captureLayout = {} as const\n",
            json_text(&layout, true, true)?
        )
        .into_bytes(),
    );
    project_schema_metadata(source, &manifest.contracts, &mut artifacts)?;
    catalog::project(source, &registry, &mut artifacts)?;
    project_fixtures(source, public_names, &mut artifacts)?;
    let bundles = Value::Object(bundles);
    let fingerprint = fingerprints(source, &manifest.contracts, &bundles, &artifacts)?;
    add_json(
        &mut artifacts,
        "contracts/fingerprints.json",
        &fingerprint,
        true,
    )?;
    Ok((artifacts, bundles))
}

fn project_schema_metadata(
    source: &Source,
    entries: &[Entry],
    artifacts: &mut Artifacts,
) -> Result<()> {
    let aliases = Value::Object(
        entries
            .iter()
            .filter(|entry| entry.is_public_schema())
            .map(|entry| {
                (
                    entry
                        .uri
                        .replacen("https://cirnet.local/", "https://netcircuit.local/", 1),
                    Value::String(entry.uri.clone()),
                )
            })
            .collect(),
    );
    let alias_source = format!(
        "// Generated schema identity compatibility aliases.\nexport const schemaUriAliases: Readonly<Record<string, string>> = {}\n\
         export function canonicalSchemaUri(uri: string): string {{\n  const index = uri.indexOf('#')\n  const document = index < 0 ? uri : uri.slice(0, index)\n  return (schemaUriAliases[document] ?? document) + (index < 0 ? '' : uri.slice(index))\n}}\n",
        json_text(&aliases, true, true)?
    );
    artifacts.insert(
        format!("{BROWSER}/generated/schemaAliases.ts"),
        alias_source.into_bytes(),
    );

    let lab_path = "contracts/physical-lab/lab.schema.json";
    let mut lab = source.json(lab_path)?;
    let lab_names: Vec<_> = entries
        .iter()
        .filter(|entry| entry.is_public_schema() && entry.path == lab_path)
        .map(|entry| entry.id.as_str())
        .collect();
    let definitions = object(&lab["$defs"], "lab schema definitions")?;
    lab["$defs"] = Value::Object(
        lab_names
            .iter()
            .map(|&name| {
                definitions
                    .get(name)
                    .map(|value| (name.to_owned(), value.clone()))
                    .ok_or_else(|| format!("Missing lab schema definition {name}"))
            })
            .collect::<std::result::Result<Map<_, _>, _>>()?,
    );
    add_json(
        artifacts,
        format!("{BROWSER}/data/labContracts.json"),
        &lab,
        true,
    )?;
    artifacts.insert(
        format!("{BROWSER}/types/lab.ts"),
        format!(
            "// Generated by scripts/generate_contracts.py; edit canonical contracts.\nexport type {{ {} }} from '../contracts/generated/dto'\n",
            lab_names.join(", ")
        )
        .into_bytes(),
    );
    Ok(())
}

fn project_fixtures(
    source: &Source,
    public_names: &Map<String, Value>,
    artifacts: &mut Artifacts,
) -> Result<()> {
    for (folder, prefix) in [
        ("contracts/fixtures", ""),
        ("contracts/design-models/fixtures", ""),
        ("contracts/fixtures/capture", "capture-"),
    ] {
        for path in json_files(source, &source.path(folder)?, false)? {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or("Non-UTF-8 fixture name")?;
            let output = format!("{BROWSER}/fixtures/{prefix}{name}");
            if folder == "contracts/fixtures" {
                let mut fixture = read_json(&source.contained(&path)?)?;
                if let Some(cases) = fixture.get_mut("cases") {
                    cases
                        .as_array_mut()
                        .ok_or("Expected fixture cases array")?
                        .retain(|case| {
                            let public = case.get("contract").is_none_or(|name| {
                                name.is_null()
                                    || name
                                        .as_str()
                                        .is_some_and(|name| public_names.contains_key(name))
                            });
                            let internal = case
                                .get("internal_only")
                                .is_some_and(|value| !value.is_null() && value != false);
                            case.is_object() && public && !internal
                        });
                }
                add_json(artifacts, output, &fixture, true)?;
            } else {
                artifacts.insert(output, normalize_lf(read(&source.contained(&path)?)?));
            }
        }
    }
    Ok(())
}

fn fingerprints(
    canonical_source: &Source,
    entries: &[Entry],
    bundles: &Value,
    artifacts: &Artifacts,
) -> Result<Value> {
    let canonical: BTreeSet<_> = entries
        .iter()
        .map(|entry| entry.path.as_str())
        .chain([MANIFEST, PROFILE_SOURCE])
        .collect();
    let mut source = Sha256::new();
    for name in canonical {
        let value = match artifacts.get(name) {
            Some(value) => value.clone(),
            None => canonical_source.read(name)?,
        };
        hash_source(&mut source, name, value);
    }
    let catalog_root = canonical_source.path("device-library")?;
    for path in json_files(canonical_source, &catalog_root, true)? {
        let relative = path.strip_prefix(&catalog_root)?;
        let name = format!(
            "device-library/{}",
            relative.to_string_lossy().replace('\\', "/")
        );
        hash_source(
            &mut source,
            &name,
            read(&canonical_source.contained(&path)?)?,
        );
    }
    let mut sorted_bundles = bundles.clone();
    sorted_bundles.sort_all_objects();
    let public = Sha256::digest(json_text(&sorted_bundles, false, true)?);
    let openapi = Sha256::digest(&artifacts[OPENAPI]);
    Ok(serde_json::json!({
        "canonical_source_sha256": format!("{:x}", source.finalize()),
        "public_schema_sha256": format!("{public:x}"),
        "openapi_sha256": format!("{openapi:x}"),
    }))
}

fn hash_source(hash: &mut Sha256, name: &str, data: Vec<u8>) {
    hash.update(name.as_bytes());
    hash.update(b"\0");
    hash.update(normalize_lf(data));
    hash.update(b"\0");
}

fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|error| format!("{}: {error}", path.display()).into())
}

fn read_json(path: &Path) -> Result<Value> {
    serde_json::from_slice(&read(path)?)
        .map_err(|error| format!("{}: {error}", path.display()).into())
}

fn json_files(source: &Source, folder: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(source.contained(folder)?)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = folder.join(entry.file_name());
        if file_type.is_dir() && recursive {
            paths.extend(json_files(source, &path, true)?);
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "json") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn normalize_lf(data: Vec<u8>) -> Vec<u8> {
    let mut bytes = data.into_iter().peekable();
    let mut normalized = Vec::with_capacity(bytes.len());
    while let Some(byte) = bytes.next() {
        if byte != b'\r' || bytes.peek() != Some(&b'\n') {
            normalized.push(byte);
        }
    }
    normalized
}

fn add_json(
    artifacts: &mut Artifacts,
    name: impl Into<String>,
    data: &Value,
    ascii: bool,
) -> Result<()> {
    let mut text = json_text(data, true, ascii)?;
    text.push('\n');
    artifacts.insert(name.into(), text.into_bytes());
    Ok(())
}

fn json_text(data: &Value, pretty: bool, ascii: bool) -> Result<String> {
    let text = if pretty {
        serde_json::to_string_pretty(data)?
    } else {
        serde_json::to_string(data)?
    };
    if !ascii {
        return Ok(text);
    }
    // The legacy fingerprint and TS projections use JSON's ASCII escape spelling.
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch < '\u{7f}' {
            escaped.push(ch);
        } else {
            for unit in ch.encode_utf16(&mut [0; 2]) {
                write!(escaped, "\\u{unit:04x}")?;
            }
        }
    }
    Ok(escaped)
}

fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| format!("Expected object for {name}").into())
}

fn array<'a>(value: &'a Value, name: &str) -> Result<&'a [Value]> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("Expected array for {name}").into())
}

fn string<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value
        .as_str()
        .ok_or_else(|| format!("Expected string for {name}").into())
}
