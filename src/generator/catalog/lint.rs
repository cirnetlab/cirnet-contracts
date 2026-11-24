use super::{DOMAINS, Result, array, object, string};
use crate::validation::Registry;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type Pins<'a> = BTreeMap<&'a str, &'a str>;
const AXES: [&str; 3] = ["x", "y", "z"];

pub(super) fn validate(registry: &Registry, catalog: &Value, mechanics: &Value) -> Result<()> {
    if catalog["schemaVersion"].as_u64() != Some(1) || catalog["units"] != "mm" {
        return Err("Unsupported physical part registry version or units".into());
    }
    let current = array(&catalog["parts"], "physical parts")?;
    let historical = optional_array(catalog, "historicalParts")?;
    let mut identities = BTreeSet::new();
    for part in current.iter().chain(historical) {
        let id = string(&part["id"], "physical part id")?;
        if !identities.insert((id, revision(&part["revision"])?)) {
            return Err(format!("Duplicate physical part identity: {id}").into());
        }
        validate_part(registry, part).map_err(|error| format!("Physical part {id}: {error}"))?;
    }
    validate_mechanics(registry, current, mechanics)
}

fn validate_part(registry: &Registry, part: &Value) -> Result<()> {
    let package = &part["package"];
    let footprint = &package["footprint"];
    revision(&package["revision"])?;
    revision(&footprint["revision"])?;
    let body = point(&package["bodyMm"])?;
    if body.iter().any(|dimension| *dimension <= 0.0) {
        return Err("Invalid package body dimensions".into());
    }
    let diameter = number(&package["leadDiameterMm"])?;
    if diameter <= 0.0
        || number(&footprint["pitchMm"])? <= 0.0
        || number(&package["standoffMm"])? < 0.0
    {
        return Err("Invalid footprint or lead dimensions".into());
    }
    let mountable = footprint["mountable"]
        .as_bool()
        .ok_or("Footprint requires explicit mounting support")?;
    let pins = validate_pins(array(&package["pins"], "package pins")?, mountable)?;
    for connection in optional_array(part, "internalConnections")? {
        let endpoints = array(&connection["pins"], "internal connection pins")?;
        if endpoints.len() != 2 || !endpoints.iter().all(|pin| known_pin(&pins, pin)) {
            return Err("Internal connection references unknown pins".into());
        }
    }
    validate_shape(registry, package, &pins, body, diameter)?;
    validate_parameters(&part["editor"])?;
    if let Some(simulation) = part.get("simulation").filter(|value| !value.is_null()) {
        validate_simulation(simulation, &pins)?;
    }
    Ok(())
}

fn validate_pins(pins: &[Value], mountable: bool) -> Result<Pins<'_>> {
    let mut roles = BTreeMap::new();
    let mut positions = Vec::new();
    if pins.is_empty() {
        return Err("Ambiguous physical pin identity".into());
    }
    for pin in pins {
        let id = string(&pin["id"], "physical pin identity")?;
        let role = string(&pin["role"], "physical pin role")?;
        if id.is_empty() || id.contains('.') || roles.insert(id, role).is_some() {
            return Err("Ambiguous physical pin identity".into());
        }
        let domain = string(&pin["domain"], "physical pin domain")?;
        if !DOMAINS.contains(&domain)
            || (role == "power" && domain != "POWER")
            || (role == "ground" && domain != "GROUND")
        {
            return Err(format!("Invalid physical pin domain: {id}").into());
        }
        let position = point(&pin["positionMm"])?;
        if !matches!(
            role,
            "input" | "output" | "power" | "ground" | "passive" | "nc"
        ) || (mountable && position[1] != 0.0)
        {
            return Err(format!("Invalid physical pin descriptor: {id}").into());
        }
        if positions.contains(&position) {
            return Err(format!("Physical pins overlap: {id}").into());
        }
        positions.push(position);
        if role == "output"
            && !matches!(
                pin["outputType"].as_str(),
                Some("push-pull" | "open-collector" | "open-drain")
            )
        {
            return Err(format!("Output pin requires its driver type: {id}").into());
        }
        validate_pin_semantics(pin, role)?;
    }
    Ok(roles)
}

fn validate_pin_semantics(pin: &Value, role: &str) -> Result<()> {
    let signal = optional_text(&pin["signal"])?;
    let polarity = optional_text(&pin["polarity"])?;
    let input = optional_text(&pin["inputCharacteristics"])?;
    let edge = optional_text(&pin["edge"])?;
    if !matches!(
        signal,
        None | Some("data" | "clock" | "set" | "reset" | "enable")
    ) || !matches!(polarity, None | Some("active-high" | "active-low"))
        || !matches!(input, None | Some("standard" | "schmitt-trigger"))
    {
        return Err("Invalid physical pin signal, polarity or input characteristic".into());
    }
    match signal {
        Some("clock") if role != "input" || !matches!(edge, Some("rising" | "falling")) => {
            return Err("Clock input requires an explicit edge".into());
        }
        Some("set" | "reset" | "enable") if role != "input" || polarity.is_none() => {
            return Err("Control input requires explicit polarity".into());
        }
        _ => {}
    }
    if edge.is_some_and(|edge| !edge.is_empty()) && signal != Some("clock") {
        return Err("Only clock inputs declare an edge".into());
    }
    if input.is_some() && role != "input" {
        return Err("Only inputs declare input characteristics".into());
    }
    Ok(())
}

fn validate_shape(
    registry: &Registry,
    package: &Value,
    pins: &Pins<'_>,
    body: [f64; 3],
    diameter: f64,
) -> Result<()> {
    let shape = &package["shape"];
    if shape.is_null() {
        return Ok(());
    }
    registry
        .validate("PackageShapeV1", "structural", shape)
        .map_err(|error| format!("Invalid package shape: {error}"))?;
    let shape_family = string(&shape["family"], "package shape family")?;
    let package_family = string(&package["family"], "mechanical package family")?;
    let compatible = matches!(
        (package_family, shape_family),
        ("axial", "resistor" | "diode" | "zener")
            | ("radial-capacitor", "ceramic" | "electrolytic")
            | ("radial-led", "led")
            | ("dip", "dip")
            | ("to92", "to92")
            | ("switch", "tactile" | "slide-switch" | "dip-switch")
            | ("seven-segment", "seven-segment")
            | ("buzzer", "buzzer")
    );
    if !compatible {
        return Err("Invalid package shape family for mechanical package".into());
    }
    let exits = array(&shape["leadExits"], "package shape lead exits")?;
    let mut ids = BTreeSet::new();
    for exit in exits {
        let id = string(&exit["pinId"], "package shape exit pin")?;
        if !pins.contains_key(id) || !ids.insert(id) {
            return Err("Package shape must seal each declared pin exactly once".into());
        }
        let [x, y, z] = point(&exit["normal"])?;
        if !close(x.hypot(y).hypot(z), 1.0) {
            return Err("Package shape exit normal must be a unit vector".into());
        }
        if point(&exit["positionMm"])?
            .iter()
            .zip(body)
            .any(|(position, size)| position.abs() > size / 2.0 + 1e-9)
        {
            return Err("Package shape exit must be within its declared envelope".into());
        }
    }
    if ids.len() != pins.len() {
        return Err("Package shape must seal each declared pin exactly once".into());
    }
    let section = &shape["leadSection"];
    let width = number(&section["widthMm"])?;
    if section["kind"] == "round"
        && (width != number(&section["thicknessMm"])? || width != diameter)
    {
        return Err("Round package section must agree with its wire diameter".into());
    }
    if section["kind"] == "stamped" && section["shoulderWidthMm"].as_f64().unwrap_or(0.0) < width {
        return Err("Stamped section requires a shoulder at least as wide as its tail".into());
    }
    validate_shape_envelope(shape_family, package, shape, body)
}

fn validate_shape_envelope(
    family: &str,
    package: &Value,
    shape: &Value,
    [x, y, z]: [f64; 3],
) -> Result<()> {
    let size = |name: &str| number(&shape[name]);
    match family {
        "led" => {
            let flange = size("flangeDiameterMm")?;
            if size("lensDiameterMm")? > flange
                || flange != x
                || flange != z
                || size("flangeHeightMm")? + size("domeRadiusMm")? > y
            {
                return Err("LED lens/flange must fit the declared shell envelope".into());
            }
        }
        "tactile" | "dip-switch" | "slide-switch" => {
            let case = size("caseHeightMm")?;
            if case > y || (family == "slide-switch" && !close(case + size("knobHeightMm")?, y)) {
                return Err("Switch case and actuator must fit the declared envelope".into());
            }
        }
        "seven-segment" => {
            let glyphs = (number(&package["digits"])? - 1.0) * size("digitPitchMm")?
                + size("digitWidthMm")?
                + size("decimalPointDiameterMm")?;
            let inset = 2.0 * size("faceInsetMm")?;
            if glyphs > x - inset || size("digitHeightMm")? > z - inset {
                return Err("Display glyph and circular DP must fit their package face".into());
            }
        }
        "buzzer" if size("openingDiameterMm")? >= x.min(z) || size("openingDepthMm")? >= y => {
            return Err("Buzzer mouth must retain an interior floor and side wall".into());
        }
        _ => {}
    }
    Ok(())
}

fn validate_mechanics(registry: &Registry, parts: &[Value], mechanics: &Value) -> Result<()> {
    let mut profiles = BTreeMap::new();
    for profile in array(&mechanics["profiles"], "mechanical package profiles")? {
        let key = (
            string(&profile["packageId"], "mechanical package id")?,
            revision(&profile["packageRevision"])?,
        );
        if profiles.insert(key, profile).is_some() {
            return Err("Ambiguous mechanical package profile".into());
        }
    }
    for part in parts {
        let package = &part["package"];
        if let Some(insertion) = package.get("boardInsertion") {
            registry
                .validate("BoardInsertionV1", "structural", insertion)
                .map_err(|error| format!("Invalid board insertion profile: {error}"))?;
        }
        let shape = &package["shape"];
        if shape.is_null()
            || array(&package["pins"], "package pins")?.len() != 2
            || !matches!(
                package["family"].as_str(),
                Some("axial" | "radial-capacitor" | "radial-led")
            )
        {
            continue;
        }
        let key = (
            string(&package["id"], "mechanical package id")?,
            revision(&package["revision"])?,
        );
        let profile = profiles
            .get(&key)
            .ok_or("Mechanical profile must use the declared shape exits")?;
        let profile_exits = exits_by_pin(&profile["pins"], "bodyExitMm")?;
        let shape_exits = exits_by_pin(&shape["leadExits"], "positionMm")?;
        if profile_exits != shape_exits {
            return Err("Mechanical profile must use the declared shape exits".into());
        }
        if shape["leadSection"]["kind"] == "round"
            && number(&package["leadDiameterMm"])? == 0.6
            && (number(&profile["minBendRadiusMm"])? < 0.4
                || !(0.6..=0.8).contains(&number(&profile["preferredBendRadiusMm"])?))
        {
            return Err("0.6 mm round leads require a nondegenerate design bend radius".into());
        }
    }
    Ok(())
}

fn exits_by_pin<'a>(exits: &'a Value, position: &str) -> Result<BTreeMap<&'a str, [f64; 3]>> {
    let mut by_pin = BTreeMap::new();
    for exit in array(exits, "mechanical exits")? {
        let id = string(&exit["pinId"], "mechanical exit pin")?;
        if by_pin.insert(id, point(&exit[position])?).is_some() {
            return Err("Ambiguous mechanical exit pin".into());
        }
    }
    Ok(by_pin)
}

fn validate_parameters(editor: &Value) -> Result<()> {
    let defaults = object(&editor["defaults"], "editor defaults")?;
    for parameter in array(&editor["parameters"], "editor parameters")? {
        let key = string(&parameter["key"], "editor parameter key")?;
        let value = defaults
            .get(key)
            .ok_or_else(|| format!("Missing editor property default: {key}"))?;
        match string(&parameter["kind"], "editor parameter kind")? {
            "boolean" if value.is_boolean() => {}
            "boolean" => return Err("Boolean property default is required".into()),
            kind @ ("enum" | "string" | "color") => {
                validate_text_parameter(parameter, value, kind)?
            }
            kind @ ("number" | "integer") => {
                let value = number(value)?;
                let min = parameter["min"].as_f64().unwrap_or(f64::NEG_INFINITY);
                let max = parameter["max"].as_f64().unwrap_or(f64::INFINITY);
                if (kind == "integer" && value.fract() != 0.0)
                    || value < min
                    || value > max
                    || (parameter["exclusiveMin"] == true && value <= min)
                {
                    return Err(
                        "Property default must be finite and within its declared range".into(),
                    );
                }
            }
            _ => return Err("Unknown editor property kind".into()),
        }
    }
    Ok(())
}

fn validate_text_parameter(parameter: &Value, value: &Value, kind: &str) -> Result<()> {
    let text = string(value, "editor text default")?;
    let mut choices = BTreeSet::new();
    for option in optional_array(parameter, "options")? {
        string(&option["label"], "editor option label")?;
        if !choices.insert(string(&option["value"], "editor option value")?) {
            return Err("Property options require unique string values and labels".into());
        }
    }
    if text.chars().count() as u64 > parameter["maxLength"].as_u64().unwrap_or(256)
        || (kind == "enum" && !choices.contains(text))
    {
        return Err("String property default must match its declared options and bounds".into());
    }
    let pattern = parameter["pattern"]
        .as_str()
        .or((kind == "color").then_some("#[0-9a-fA-F]{6}"));
    if let Some(pattern) = pattern.filter(|pattern| !pattern.is_empty()) {
        let schema = json!({"type": "string", "pattern": format!("^(?:{pattern})(?![\\s\\S])")});
        if !jsonschema::draft202012::new(&schema)?.is_valid(value) {
            return Err("String property default must match its declared pattern".into());
        }
    }
    Ok(())
}

fn validate_simulation(simulation: &Value, pins: &Pins<'_>) -> Result<()> {
    let power = &simulation["power"];
    if power.as_object().is_some_and(|power| !power.is_empty()) {
        let positive = string(&power["positivePinId"], "positive power pin")?;
        let negative = string(&power["negativePinId"], "negative power pin")?;
        let min = number(&power["minVoltage"])?;
        let max = number(&power["maxVoltage"])?;
        if pins.get(positive) != Some(&"power")
            || pins.get(negative) != Some(&"ground")
            || !(0.0 < min && min < max)
        {
            return Err("Power requirement has invalid pins, roles or envelope".into());
        }
    }
    match simulation["status"].as_str() {
        Some("supported") => {}
        Some("unsupported")
            if simulation["reason"]
                .as_str()
                .is_some_and(|reason| !reason.is_empty()) => {}
        _ => return Err("Simulation support must be explicit".into()),
    }
    for gate in optional_array(simulation, "gates")? {
        let pins_used = array(&gate["inputs"], "gate inputs")?;
        if !pins_used
            .iter()
            .chain([&gate["output"]])
            .all(|pin| known_pin(pins, pin))
        {
            return Err("Gate references unknown pins".into());
        }
    }
    Ok(())
}

fn optional_array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    value.get(key).map_or(Ok(&[]), |value| array(value, key))
}

fn optional_text(value: &Value) -> Result<Option<&str>> {
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(string(value, "optional pin descriptor")?))
    }
}

fn known_pin(pins: &Pins<'_>, value: &Value) -> bool {
    value.as_str().is_some_and(|id| pins.contains_key(id))
}

fn revision(value: &Value) -> Result<u64> {
    value
        .as_u64()
        .filter(|revision| *revision > 0)
        .ok_or_else(|| "Physical part and package require positive revisions".into())
}

fn number(value: &Value) -> Result<f64> {
    value
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| "Invalid finite physical catalog dimension or quantity".into())
}

fn point(value: &Value) -> Result<[f64; 3]> {
    let [x, y, z] = AXES;
    Ok([number(&value[x])?, number(&value[y])?, number(&value[z])?])
}

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9 * left.abs().max(right.abs()).max(1.0)
}
