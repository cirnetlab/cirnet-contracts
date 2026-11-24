use super::{Registry, RegistryError, ValidationError, ValidationIssue, child, numeric, read_json};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

mod geometry;

const MM_TO_WORLD: f64 = 0.13 / 2.54;
const EPS: f64 = 0.00001;
type DesignResult<T> = Result<T, String>;

pub(super) struct Catalogs {
    ports: HashMap<String, HashMap<String, Value>>,
    boards: HashMap<String, Board>,
    current_parts: Vec<Value>,
    historical_parts: Vec<Value>,
    flexible_profiles: Vec<Value>,
    board_mechanics: Value,
}

struct Board {
    contacts: HashMap<String, Value>,
    deck_height: f64,
}

impl Catalogs {
    pub(super) fn load(root: &Path) -> Result<Self, RegistryError> {
        let signal: Value = read_json(root, "contracts/design-models/signal-port-catalog.json")?;
        let boards: Value = read_json(root, "contracts/design-models/breadboards.json")?;
        let parts: Value = read_json(root, "contracts/design-models/physical-parts.v1.json")?;
        let flexible: Value = read_json(root, "contracts/design-models/flexible-mechanics.json")?;
        let board_mechanics = read_json(root, "contracts/design-models/breadboard-mechanics.json")?;
        let ports = catalog_entries(&signal, "components")?
            .iter()
            .map(|component| {
                Ok((
                    catalog_text(component, "type")?.to_owned(),
                    catalog_entries(component, "ports")?
                        .iter()
                        .map(|port| {
                            Ok((catalog_text(port, "id")?.to_owned(), port["domain"].clone()))
                        })
                        .collect::<Result<_, RegistryError>>()?,
                ))
            })
            .collect::<Result<_, RegistryError>>()?;
        let mut board_types = HashMap::new();
        for board in catalog_entries(&boards, "profiles")? {
            let contacts = catalog_entries(board, "contacts")?
                .iter()
                .map(|contact| {
                    Ok((
                        catalog_text(contact, "contactId")?.to_owned(),
                        contact.clone(),
                    ))
                })
                .collect::<Result<HashMap<_, _>, RegistryError>>()?;
            let deck_height = board["deckHeight"]
                .as_f64()
                .ok_or_else(|| RegistryError("Invalid breadboard deck height".into()))?;
            for module in catalog_entries(board, "moduleTypes")? {
                let kind = module
                    .as_str()
                    .ok_or_else(|| RegistryError("Invalid breadboard type".into()))?;
                board_types.insert(
                    kind.into(),
                    Board {
                        contacts: contacts.clone(),
                        deck_height,
                    },
                );
            }
        }
        Ok(Self {
            ports,
            boards: board_types,
            current_parts: catalog_entries(&parts, "parts")?.to_vec(),
            historical_parts: catalog_entries(&parts, "historicalParts")?.to_vec(),
            flexible_profiles: catalog_entries(&flexible, "profiles")?.to_vec(),
            board_mechanics,
        })
    }

    fn physical_part<'a>(&'a self, module: &Value) -> Option<&'a Value> {
        let kind = &module["type"];
        if let Some(mount) = module
            .get("mount")
            .or_else(|| module.get("physicalDescriptor"))
        {
            if module.get("mount").is_none() && !same_revision(&mount["version"], &Value::from(1)) {
                return None;
            }
            self.current_parts
                .iter()
                .chain(&self.historical_parts)
                .find(|part| {
                    &part["editorType"] == kind
                        && part["id"] == mount["partId"]
                        && same_revision(&part["revision"], &mount["partRevision"])
                        && part["package"]["id"] == mount["packageId"]
                        && same_revision(&part["package"]["revision"], &mount["packageRevision"])
                })
        } else {
            self.historical_parts
                .iter()
                .filter(|part| &part["editorType"] == kind)
                .min_by(|a, b| numeric::compare(&a["revision"], &b["revision"]))
                .or_else(|| {
                    self.current_parts
                        .iter()
                        .find(|part| &part["editorType"] == kind)
                })
        }
    }

    fn board(&self, module: &Value) -> Option<&Board> {
        self.boards.get(module["type"].as_str()?)
    }

    fn contact<'a>(&'a self, board: &Value, contact: &Value) -> Option<&'a Value> {
        self.board(board)?.contacts.get(contact.as_str()?)
    }

    fn flexible_profile(&self, package: &Value) -> Option<&Value> {
        self.flexible_profiles.iter().find(|profile| {
            profile["packageId"] == package["id"]
                && same_revision(&profile["packageRevision"], &package["revision"])
        })
    }
}

fn same_revision(a: &Value, b: &Value) -> bool {
    a.is_number() && b.is_number() && numeric::compare(a, b).is_eq()
}

fn catalog_entries<'a>(value: &'a Value, name: &str) -> Result<&'a [Value], RegistryError> {
    value[name]
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| RegistryError(format!("Expected catalog array {name}")))
}

fn catalog_text<'a>(value: &'a Value, name: &str) -> Result<&'a str, RegistryError> {
    value[name]
        .as_str()
        .ok_or_else(|| RegistryError(format!("Expected catalog string {name}")))
}

impl Registry {
    pub(super) fn design_annotations(
        &self,
        name: &str,
        profile: &str,
        value: &Value,
        path: &str,
    ) -> Result<(), ValidationError> {
        if matches!(name, "SimulationRequest" | "SimulationRequestV2") {
            return self.design_annotations(
                "CircuitGraph",
                profile,
                &value["graph"],
                &child(path, "graph"),
            );
        }
        let design = self.catalogs()?;
        let graph = matches!(
            name,
            "CircuitGraph" | "CircuitGraphV12" | "AnySupportedCircuitGraph"
        );
        let modules = if graph {
            list(&value["modules"])
        } else {
            std::slice::from_ref(value)
        };
        let path_for = |index| {
            if graph {
                child(&child(path, "modules"), index)
            } else {
                path.to_owned()
            }
        };
        let by_id: HashMap<_, _> = modules
            .iter()
            .map(|module| (&module["id"], module))
            .map(|(id, module)| (id.as_str().expect("validated module ID"), module))
            .collect();
        let mut labels = HashSet::new();
        for (index, module) in modules.iter().enumerate() {
            if module["type"] != "LABEL" {
                continue;
            }
            labels.insert(module["id"].as_str().expect("validated module ID"));
            let location = path_for(index);
            if module.get("mount").is_some() || module.get("physicalDescriptor").is_some() {
                return Err(ValidationIssue::new(
                    "INVALID_LABEL_MOUNT", &location,
                    "A Label has only a visual board association and cannot have physical mounting.", profile,
                ).into());
            }
            let properties = child(&location, "properties");
            let empty = Value::Object(serde_json::Map::new());
            let value = module.get("properties").unwrap_or(&empty);
            self.validate_at("LabelPropertiesV1", profile, value, &properties)?;
            if let Some(association) = value["boardId"].as_str() {
                if graph
                    && by_id
                        .get(association)
                        .and_then(|board| design.board(board))
                        .is_none()
                {
                    return Err(ValidationIssue::new(
                        "INVALID_LABEL_BOARD",
                        &child(&properties, "boardId"),
                        "A Label board association must reference an existing breadboard.",
                        profile,
                    )
                    .into());
                }
            }
        }
        if graph && !labels.is_empty() {
            for (index, edge) in list(&value["connections"]).iter().enumerate() {
                if ["source", "destination"].iter().any(|key| {
                    edge[*key]
                        .as_str()
                        .and_then(|end| end.split('.').next())
                        .is_some_and(|id| labels.contains(id))
                }) {
                    return Err(ValidationIssue::new(
                        "INVALID_LABEL_CONNECTION",
                        &child(&child(path, "connections"), index),
                        "A Label has no electrical ports or nets.",
                        profile,
                    )
                    .into());
                }
            }
        }
        for (index, module) in modules.iter().enumerate() {
            let Some(namespace) = module.get("signal_domains") else {
                continue;
            };
            let location = child(&path_for(index), "signal_domains");
            let Some(version) = namespace["version"].as_str() else {
                return Err(ValidationIssue::new(
                    "INVALID_SIGNAL_DOMAINS",
                    &location,
                    "Signal domains require an explicit namespace version.",
                    profile,
                )
                .into());
            };
            if version != "1.0" {
                continue;
            }
            self.validate_at("SignalDomainNamespace", profile, namespace, &location)?;
            let Some(ports) = design.ports.get(text(&module["type"])) else {
                continue;
            };
            for (port, domain) in namespace["ports"].as_object().expect("validated namespace") {
                let path = child(&child(&location, "ports"), port);
                let Some(canonical) = ports.get(port) else {
                    return Err(ValidationIssue::new(
                        "UNKNOWN_SIGNAL_PORT",
                        &path,
                        "The namespace references an unknown catalog port.",
                        profile,
                    )
                    .into());
                };
                if domain != canonical {
                    return Err(ValidationIssue::new(
                        "SIGNAL_DOMAIN_CONFLICT",
                        &path,
                        "An instance annotation cannot relabel the canonical port domain.",
                        profile,
                    )
                    .into());
                }
            }
        }
        Ok(())
    }

    pub(super) fn physical_design(
        &self,
        graph: &Value,
        profile: &str,
        path: &str,
    ) -> Result<(), ValidationError> {
        let design = self.catalogs()?;
        self.check_physical_design(graph, design)
            .map_err(|message| {
                ValidationIssue::new("INVALID_REFERENCE_DESIGN", path, message, profile).into()
            })
    }

    fn check_physical_design(&self, graph: &Value, design: &Catalogs) -> DesignResult<()> {
        let modules = list(&graph["modules"]);
        for module in modules {
            self.component_properties(module)?;
        }
        let physical: Vec<_> = modules
            .iter()
            .map(|module| design.physical_part(module))
            .collect();
        if !modules.iter().zip(&physical).any(|(module, part)| {
            part.is_some()
                || module.get("mount").is_some()
                || module.get("physicalDescriptor").is_some()
        }) {
            return Ok(());
        }
        let version = text(&graph["schema_version"]);
        if !matches!(version, "1.1" | "1.2") {
            return Err("Physical packages and mounting require CircuitGraph 1.1 or 1.2".into());
        }
        let connections = list(&graph["connections"]);
        if modules.len() > 2000 || connections.len() > 10000 {
            return Err("Physical design graph exceeds supported limits".into());
        }
        let by_id: HashMap<_, _> = modules
            .iter()
            .map(|module| (text(&module["id"]), module))
            .collect();
        if by_id.len() != modules.len() || by_id.keys().any(|id| id.contains('.')) {
            return Err("Duplicate or invalid module id".into());
        }

        let mut occupied = HashSet::new();
        let mut flexible = HashMap::new();
        for (module, part) in modules.iter().zip(&physical) {
            if module.get("physicalDescriptor").is_some() && (version != "1.2" || part.is_none()) {
                return Err(
                    "Free physical descriptors require CircuitGraph 1.2 and a canonical package"
                        .into(),
                );
            }
            let Some(mount) = module.get("mount") else {
                continue;
            };
            let part = part.ok_or("Mount requires a canonical physical part")?;
            let package = &part["package"];
            let footprint = &package["footprint"];
            if footprint["mountable"] != true {
                return Err("Package does not have a breadboard mounting footprint".into());
            }
            let board_id = text(&mount["boardId"]);
            let board = by_id
                .get(board_id)
                .copied()
                .filter(|board| design.board(board).is_some() && board.get("mount").is_none())
                .ok_or("Unknown mounting breadboard")?;
            if module.get("position").is_some() || module.get("rotation").is_some() {
                return Err(
                    "Mounted world pose must be derived from the authoritative relative pose"
                        .into(),
                );
            }
            let pins: HashMap<_, _> = list(&package["pins"])
                .iter()
                .map(|pin| (text(&pin["id"]), pin))
                .collect();
            let bindings = list(&mount["contacts"]);
            let bound_pins: HashSet<_> = bindings
                .iter()
                .map(|binding| text(&binding["pinId"]))
                .collect();
            if bindings.len() != pins.len()
                || bound_pins.len() != pins.len()
                || !pins.keys().all(|pin| bound_pins.contains(pin))
            {
                return Err("Mount must bind each package pin exactly once".into());
            }
            if mount["kind"] == "breadboard-flexible" {
                if version != "1.2" {
                    return Err("Flexible assemblies require CircuitGraph 1.2".into());
                }
                let assembly = geometry::resolve_flexible(design, package, mount, board)?;
                for binding in bindings {
                    if !occupied.insert((board_id, text(&binding["contactId"]))) {
                        return Err("Breadboard contact occupancy conflict".into());
                    }
                }
                flexible.insert(text(&module["id"]), assembly);
                continue;
            }
            let pose = &mount["pose"];
            let position = geometry::point(&pose["position"]);
            let rotation = geometry::number(&pose["rotation"]);
            if !position.is_finite() || !rotation.is_finite() {
                return Err("Mount pose must be finite".into());
            }
            let yaw = geometry::wrapped_yaw(&pose["rotation"]);
            let bridge = footprint["bridgeCenterGap"] == true;
            if bridge
                && (yaw.min((yaw - 180.0).abs()).min(360.0 - yaw) > EPS || position.z().abs() > EPS)
            {
                return Err(
                    "DIP footprint must align with and bridge the breadboard center gap".into(),
                );
            }
            let mut rows = HashSet::new();
            for binding in bindings {
                let contact = design
                    .contact(board, &binding["contactId"])
                    .ok_or("Unknown mounting breadboard contact")?;
                if !occupied.insert((board_id, text(&binding["contactId"]))) {
                    return Err("Breadboard contact occupancy conflict".into());
                }
                let pin = pins[text(&binding["pinId"])];
                let expected = position
                    + geometry::yaw(yaw, geometry::point(&pin["positionMm"]) * MM_TO_WORLD);
                if expected.distance(geometry::coordinates(&contact["localPosition"])) >= EPS {
                    return Err("Pin/contact binding does not match package footprint pose".into());
                }
                rows.insert(
                    text(&contact["nodeId"])
                        .strip_prefix("terminal:")
                        .and_then(|node| node.split(':').next())
                        .unwrap_or("rail"),
                );
            }
            if bridge && rows != HashSet::from(["AE", "FJ"]) {
                return Err("DIP footprint must bridge the breadboard center gap".into());
            }
        }
        geometry::collisions(design, modules, &physical, &by_id, &flexible)?;
        for edge in connections {
            for key in ["source", "destination"] {
                let (id, token) = text(&edge[key])
                    .split_once('.')
                    .ok_or("Unknown endpoint module")?;
                let module = by_id.get(id).copied().ok_or("Unknown endpoint module")?;
                if let Some(part) = design.physical_part(module) {
                    if !list(&part["package"]["pins"])
                        .iter()
                        .any(|pin| pin["id"] == token)
                    {
                        return Err("Unknown physical package pin".into());
                    }
                }
                if let Some(contact) = token.strip_prefix("contact:") {
                    if design
                        .board(module)
                        .is_none_or(|board| !board.contacts.contains_key(contact))
                    {
                        return Err("Unknown breadboard contact".into());
                    }
                }
            }
        }
        Ok(())
    }

    fn component_properties(&self, module: &Value) -> DesignResult<()> {
        let kind = text(&module["type"]);
        let empty = Value::Object(serde_json::Map::new());
        let properties = module.get("properties").unwrap_or(&empty);
        if matches!(
            kind,
            "GENERIC_RESISTOR"
                | "GENERIC_CAPACITOR"
                | "GENERIC_CAPACITOR_ELECTROLYTIC"
                | "GENERIC_DIODE"
                | "GENERIC_ZENER_DIODE"
                | "GENERIC_LED"
                | "GENERIC_BUZZER"
                | "GENERIC_BJT"
        ) {
            self.validate_at("ComponentDesignPropertiesV1", "structural", properties, "")
                .map_err(|error| match error {
                    ValidationError::Invalid(issue) => issue.message,
                    _ => error.to_string(),
                })?;
            let expected = match kind {
                "GENERIC_CAPACITOR" => Some("CERAMIC"),
                "GENERIC_CAPACITOR_ELECTROLYTIC" => Some("ELECTROLYTIC"),
                _ => None,
            };
            if let Some(expected) = expected {
                if properties
                    .get("capacitorType")
                    .is_some_and(|value| value != expected)
                {
                    return Err(
                        "Capacitor Type requires an atomic descriptor and pin mapping change"
                            .into(),
                    );
                }
            }
        }

        let limits: &[(&str, f64, f64, &str)] = match kind {
            "RESISTOR_AXIAL" => &[
                ("resistanceOhms", 0.1, 1e9, "0.1 and 1000000000.0"),
                ("powerWatts", 0.001, 100.0, "0.001 and 100"),
                ("tolerancePercent", 0.001, 20.0, "0.001 and 20"),
            ],
            "CAPACITOR_CERAMIC" | "CAPACITOR_ELECTROLYTIC" => &[
                ("capacitanceFarads", 1e-12, 1.0, "1e-12 and 1"),
                ("voltageRatingVolts", 0.1, 10000.0, "0.1 and 10000"),
            ],
            _ => &[],
        };
        for &(key, low, high, legacy_bounds) in limits {
            let present = properties.get(key).or_else(|| {
                if key == "resistanceOhms" {
                    properties.get("resistance_ohm")
                } else {
                    None
                }
            });
            if let Some(value) = present {
                if !value
                    .as_f64()
                    .is_some_and(|value| (low..=high).contains(&value))
                {
                    return Err(format!(
                        "{key} must be a finite SI number between {legacy_bounds}"
                    ));
                }
            }
        }
        Ok(())
    }
}

fn list(value: &Value) -> &[Value] {
    value.as_array().expect("schema-validated array")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("schema-validated string")
}
