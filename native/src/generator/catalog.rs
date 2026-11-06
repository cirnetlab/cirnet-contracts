use super::{
    Artifacts, BROWSER, PROJECTED_CATALOGS, Result, Source, add_json, array, normalize_lf, object,
    string,
};
use crate::validation::Registry;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
mod lint;

const PART_KEYS: &[&str] = &[
    "id",
    "revision",
    "editorType",
    "name",
    "shortName",
    "function",
    "package",
    "editor",
    "internalConnections",
    "aliases",
    "logic",
    "identityKind",
];
const DOMAINS: &[&str] = &["DIGITAL", "ANALOG", "POWER", "GROUND", "CLOCK", "MIXED"];
const DIGITAL_TYPES: &[&str] = &[
    "DIGITAL_SWITCH",
    "DIGITAL_INPUT",
    "INPUT",
    "CLOCK",
    "LED",
    "PROBE",
    "OUTPUT",
    "74HC08",
    "AND",
    "OR",
    "XOR",
    "NOT",
];
const DC_MODELS: &[&str] = &[
    "dc-source",
    "hc08-powered",
    "resistor-dc",
    "led-dc",
    "contact-switch",
];

pub(super) fn project(
    source: &Source,
    registry: &Registry,
    artifacts: &mut Artifacts,
) -> Result<()> {
    let mut editor = source.json("device-library/editor/components.json")?;
    let parts_bytes = normalize_lf(source.read("device-library/parts/physical-parts.json")?);
    let parts: Value = serde_json::from_slice(&parts_bytes)?;
    let mechanics = source.json("contracts/design-models/flexible-mechanics.json")?;
    lint::validate(registry, &parts, &mechanics)?;

    let mut models = BTreeMap::new();
    let mut physical = Vec::new();
    for part in array(&parts["parts"], "physical parts")? {
        let kind = string(&part["editorType"], "physical part editor type")?;
        if models.insert(kind, &part["simulation"]).is_some() {
            return Err(format!("Duplicate physical part editor type: {kind}").into());
        }
        physical.push(component(part, kind)?);
    }
    editor["components"]
        .as_array_mut()
        .ok_or("Expected editor components array")?
        .extend(physical);

    let mut domains = Vec::new();
    let mut types = BTreeSet::new();
    for component in array(&editor["components"], "editor components")? {
        let kind = string(&component["type"], "component type")?;
        let execution = execution(
            kind,
            component["visualOnly"].as_bool().unwrap_or(false),
            models.get(kind).copied(),
        );
        append_domain(
            &mut domains,
            &mut types,
            kind,
            &component["ports"],
            execution,
        )?;
    }
    if let Some(legacy) = editor.get("legacyPorts") {
        for (kind, ports) in object(legacy, "legacy ports")? {
            append_domain(
                &mut domains,
                &mut types,
                kind,
                ports,
                execution(kind, false, None),
            )?;
        }
    }
    let domains = json!({"version": "1.0", "components": domains});
    let mut editor_parts =
        json!({"schemaVersion": parts["schemaVersion"], "units": parts["units"]});
    for collection in ["parts", "historicalParts"] {
        let values = match parts.get(collection) {
            Some(value) => array(value, collection)?,
            None => &[],
        };
        let parts = values
            .iter()
            .map(|part| {
                Ok(Value::Object(
                    object(part, "editor part")?
                        .iter()
                        .filter(|(key, _)| PART_KEYS.contains(&key.as_str()))
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect::<Map<_, _>>(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        editor_parts[collection] = Value::Array(parts);
    }

    let board_mechanics = source.json("contracts/design-models/breadboard-mechanics.json")?;
    registry.validate(
        "BreadboardMechanicalProfileV1",
        "structural",
        &board_mechanics,
    )?;

    for (name, value) in [
        ("editorCatalog", &editor),
        ("editorParts", &editor_parts),
        ("physicalParts", &parts),
        ("signalPortCatalog", &domains),
        ("flexibleMechanics", &mechanics),
        ("breadboardMechanics", &board_mechanics),
    ] {
        add_json(
            artifacts,
            format!("{BROWSER}/data/{name}.json"),
            value,
            false,
        )?;
    }
    let boards_bytes = normalize_lf(source.read("contracts/design-models/breadboards.json")?);
    let boards: Value = serde_json::from_slice(&boards_bytes)?;
    validate_topologies(&boards)?;
    artifacts.insert(
        format!("{BROWSER}/data/breadboardTopologies.json"),
        boards_bytes,
    );

    // The source fingerprint includes both canonical Device Library inputs and
    // the legacy manifest's derived registry paths.
    let [physical_parts, signal_ports] = PROJECTED_CATALOGS;
    artifacts.insert(physical_parts.into(), parts_bytes);
    add_json(artifacts, signal_ports, &domains, false)?;
    Ok(())
}

fn component(part: &Value, kind: &str) -> Result<Value> {
    let package = &part["package"];
    let editor = &part["editor"];
    let scale = 0.13 / 2.54;
    let mut size = Vec::new();
    for axis in ["x", "y", "z"] {
        let number = package["bodyMm"][axis]
            .as_f64()
            .filter(|n| n.is_finite())
            .ok_or_else(|| format!("Invalid body size for {kind}: {axis}"))?;
        size.push(number * scale);
    }
    let mut ports = Vec::new();
    for pin in array(&package["pins"], "physical package pins")? {
        let direction = match string(&pin["role"], "physical pin role")? {
            role @ ("input" | "output") => role,
            "power" | "ground" | "passive" | "nc" => "inout",
            role => return Err(format!("Unknown physical pin role: {role}").into()),
        };
        ports.push(json!({
            "id": string(&pin["id"], "physical pin identity")?,
            "direction": direction,
            "domain": pin["domain"],
        }));
    }
    Ok(json!({
        "type": kind,
        "name": string(&part["name"], "physical part name")?,
        "mode": "physical",
        "family": string(&editor["family"], "physical part editor family")?,
        "visual": string(&editor["visual"], "physical part editor visual")?,
        "size": size,
        "color": string(&editor["color"], "physical part editor color")?,
        "partDefinitionId": string(&part["id"], "physical part identity")?,
        "partRevision": part.get("revision").ok_or("Missing physical part revision")?,
        "ports": ports,
        "defaults": object(&editor["defaults"], "physical part editor defaults")?,
        "parameters": array(&editor["parameters"], "physical part editor parameters")?,
    }))
}

fn execution(kind: &str, visual_only: bool, simulation: Option<&Value>) -> &'static str {
    if simulation.is_some_and(|model| {
        model["status"] == "supported"
            && model["model"]
                .as_str()
                .is_some_and(|name| DC_MODELS.contains(&name))
    }) {
        "DC_DIGITAL"
    } else if DIGITAL_TYPES.contains(&kind) {
        "DIGITAL"
    } else if visual_only {
        "VISUAL"
    } else {
        "UNAVAILABLE"
    }
}

fn append_domain<'a>(
    domains: &mut Vec<Value>,
    types: &mut BTreeSet<&'a str>,
    kind: &'a str,
    ports: &Value,
    execution: &str,
) -> Result<()> {
    if !types.insert(kind) {
        return Err(format!("Duplicate signal-domain component type: {kind}").into());
    }
    for port in array(ports, "component ports")? {
        if !port["domain"]
            .as_str()
            .is_some_and(|domain| DOMAINS.contains(&domain))
        {
            return Err(format!("Unknown signal domain for {kind}").into());
        }
    }
    domains.push(json!({"type": kind, "ports": ports, "execution": execution}));
    Ok(())
}

fn validate_topologies(boards: &Value) -> Result<()> {
    let mut module_types = BTreeSet::new();
    for profile in array(&boards["profiles"], "breadboard profiles")? {
        for kind in array(&profile["moduleTypes"], "breadboard module types")? {
            let kind = string(kind, "breadboard module type")?;
            if !module_types.insert(kind) {
                return Err(format!("Duplicate breadboard module type: {kind}").into());
            }
        }
        let mut ids = BTreeSet::new();
        let mut terminals = BTreeMap::new();
        for contact in array(&profile["contacts"], "breadboard contacts")? {
            let id = string(&contact["contactId"], "breadboard contact identity")?;
            let node = string(&contact["nodeId"], "breadboard contact node")?;
            let position = array(&contact["localPosition"], "breadboard contact position")?;
            if id.contains('.')
                || !ids.insert(id)
                || node.is_empty()
                || position.len() != 3
                || position
                    .iter()
                    .any(|coordinate| !coordinate.as_f64().is_some_and(f64::is_finite))
            {
                return Err(format!("Invalid breadboard contact descriptor: {id}").into());
            }
            if node.starts_with("terminal:") {
                *terminals.entry(node).or_insert(0) += 1;
            }
        }
        if terminals.values().any(|count| *count != 5) {
            return Err("Breadboard terminal nodes must contain five holes".into());
        }
    }
    Ok(())
}
