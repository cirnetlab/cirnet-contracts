use super::{Registry, ValidationError, ValidationIssue, child, numeric};
use num_bigint::BigInt;
use serde_json::Value;
use std::{cmp::Ordering, collections::HashSet};

impl Registry {
    pub(super) fn semantics(
        &self,
        name: &str,
        profile: &str,
        value: &Value,
        path: &str,
    ) -> Result<(), ValidationError> {
        let at = Check { profile, path };
        let nested =
            |name, field| self.semantics(name, profile, &value[field], &child(path, field));
        match name {
            "CircuitGraph"
            | "CircuitGraphV12"
            | "AnySupportedCircuitGraph"
            | "CircuitModule"
            | "CircuitModuleV12"
            | "SimulationRequest"
            | "SimulationRequestV2" => {
                self.design_annotations(name, profile, value, path)?;
            }
            "MemoryImage" => {
                if count_differs(array(&value["data"]).len(), &value["depth"]) {
                    return Err(at
                        .fail(
                            "MEMORY_DEPTH_MISMATCH",
                            "data",
                            "Memory image data length differs from depth",
                        )
                        .into());
                }
            }
            "AssignmentAuthoringCreate"
            | "AssignmentAuthoringSave"
            | "AssignmentAuthoringDocument"
            | "AssignmentAuthoringCreateV2"
            | "AssignmentAuthoringSaveV2"
            | "AssignmentAuthoringDocumentV2"
            | "AssignmentAuthoringSummary"
            | "AssignmentAuthoringSummaryV2" => {
                let document = matches!(
                    name,
                    "AssignmentAuthoringDocument" | "AssignmentAuthoringDocumentV2"
                );
                if !matches!(
                    name,
                    "AssignmentAuthoringSummary" | "AssignmentAuthoringSummaryV2"
                ) {
                    nested(assignment_definition(&value["definition"]), "definition")?;
                }
                if document
                    || matches!(
                        name,
                        "AssignmentAuthoringSummary" | "AssignmentAuthoringSummaryV2"
                    )
                {
                    let selected = &value["published_revision"];
                    if selected.is_null() == (value["published"] == true)
                        || greater(selected, &value["revision"])
                    {
                        return Err(at
                            .fail(
                                "ASSIGNMENT_PUBLICATION_MISMATCH",
                                "published_revision",
                                "published revision is inconsistent with the authoring head",
                            )
                            .into());
                    }
                }
                if document {
                    let definition = child(path, "definition");
                    for (key, code, message) in [
                        (
                            "assignment_id",
                            "ASSIGNMENT_ID_MISMATCH",
                            "assignment identity differs from its definition",
                        ),
                        (
                            "title",
                            "ASSIGNMENT_TITLE_MISMATCH",
                            "assignment title differs from its definition",
                        ),
                    ] {
                        if value[key] != value["definition"][key] {
                            return Err(ValidationIssue::new(
                                code,
                                &child(&definition, key),
                                message,
                                profile,
                            )
                            .into());
                        }
                    }
                }
            }
            "AssignmentAuthoringPage" | "AssignmentAuthoringPageV2" => {
                let summary = if name.ends_with("V2") {
                    "AssignmentAuthoringSummaryV2"
                } else {
                    "AssignmentAuthoringSummary"
                };
                for (index, item) in array(&value["items"]).iter().enumerate() {
                    self.semantics(summary, profile, item, &child(&child(path, "items"), index))?;
                }
            }
            "AnySupportedAssignmentDefinition" => {
                self.semantics(assignment_definition(value), profile, value, path)?;
            }
            "AssignmentViewV2" | "InstructorAssignmentDocumentV2"
                if value["document_kind"] == "instructor" =>
            {
                self.semantics(assignment_definition(value), profile, value, path)?;
            }
            "AssignmentDefinition" | "AssignmentDefinitionV2" => {
                let endpoints = Check {
                    path: &child(at.path, "lab_endpoints"),
                    ..at
                };
                let entries = array(&value["lab_endpoints"]);
                unique(entries, "endpoint_id", endpoints)?;
                for (index, endpoint_value) in entries.iter().enumerate() {
                    endpoint(
                        endpoint_value,
                        Check {
                            profile,
                            path: &child(endpoints.path, index),
                        },
                    )?;
                }
                if value.get("reference_topology_graph").is_some() {
                    nested("DetectedTopologyGraph", "reference_topology_graph")?;
                }
                if let Some(graph) = value.get("reference_design_graph") {
                    let location = child(path, "reference_design_graph");
                    if name == "AssignmentDefinition" {
                        self.validate_at("CircuitGraph", profile, graph, &location)?;
                    }
                    self.validate_at("AnySupportedCircuitGraph", "structural", graph, &location)?;
                    self.physical_design(graph, profile, &location)?;
                }
                if let Some(profiles) = value.get("instructor_pin_profiles") {
                    let profiles_at = Check {
                        path: &child(at.path, "instructor_pin_profiles"),
                        ..at
                    };
                    let profiles = array(profiles);
                    unique(profiles, "profile_id", profiles_at)?;
                    for (index, entry) in profiles.iter().enumerate() {
                        let location = child(&child(profiles_at.path, index), "pins");
                        unique(
                            array(&entry["pins"]),
                            "pin_id",
                            Check {
                                profile,
                                path: &location,
                            },
                        )?;
                    }
                }
            }
            "LabEndpoint" => endpoint(value, at)?,
            "ObservedPhysicalComponent"
            | "ObservedComponent"
            | "OpaquePhysicalComponent"
            | "OpaqueComponent" => {
                component(value, at)?;
            }
            "DetectedTopologyGraph" => topology(value, at)?,
            "TopologyResolutionResult" => {
                unique(
                    array(&value["endpoint_bindings"]),
                    "endpoint_id",
                    Check {
                        path: &child(at.path, "endpoint_bindings"),
                        ..at
                    },
                )?;
                for key in ["reference_component_id", "observed_component_id"] {
                    unique(
                        array(&value["component_matches"]),
                        key,
                        Check {
                            path: &child(at.path, "component_matches"),
                            ..at
                        },
                    )?;
                }
            }
            "InstrumentControlAccess" => {
                let ready = value["reason"] == "READY";
                if (value["can_configure_generator"] == true) != ready
                    || array(&value["generator_channels"]).is_empty() == ready
                {
                    return Err(at
                        .fail(
                            "CONTROL_ACCESS_MISMATCH",
                            "can_configure_generator",
                            "Control access requires ready allocated context",
                        )
                        .into());
                }
            }
            "InstrumentCommandRecord" => acknowledgement(value, at)?,
            "ExperimentHistoryPage" | "ExperimentActiveSnapshot" => {
                let items = array(&value["items"]);
                let items_at = Check {
                    path: &child(at.path, "items"),
                    ..at
                };
                unique(items, "experiment_id", items_at)?;
                for (index, entry) in items.iter().enumerate() {
                    self.semantics(
                        "ExperimentRecord",
                        profile,
                        entry,
                        &child(items_at.path, index),
                    )?;
                }
            }
            "ExperimentRecord" => {
                if array(&value["state_history"]).last() != value.get("state") {
                    return Err(at
                        .fail(
                            "STATE_HISTORY_MISMATCH",
                            "state_history",
                            "Experiment state/history disagree",
                        )
                        .into());
                }
                if !value["capture"].is_null() {
                    nested("CaptureMetadata", "capture")?;
                }
            }
            "CaptureResult" => {
                let samples = array(&value["samples"]);
                if count_differs(samples.len(), &value["sample_count"]) {
                    return Err(at
                        .fail(
                            "CAPTURE_COUNT_MISMATCH",
                            "sample_count",
                            "Capture metadata does not match samples",
                        )
                        .into());
                }
                for (index, sample) in samples.iter().enumerate() {
                    if greater(&sample["time_ns"], &value["duration_ns"]) {
                        let sample = child(&child(path, "samples"), index);
                        return Err(ValidationIssue::new(
                            "CAPTURE_TIME_OUTSIDE_DURATION",
                            &child(&sample, "time_ns"),
                            "Capture sample exceeds declared duration",
                            profile,
                        )
                        .into());
                    }
                }
            }
            "AnalogCaptureDescriptor"
            | "AnalogCaptureHeader"
            | "AnalogCaptureChannel"
            | "AnalogCaptureQuality"
            | "AnalogCaptureTimeAxis" => analog_capture(name, value, at)?,
            "CaptureMetadata" => capture_metadata(value, at)?,
            "StationCapabilities" | "PublicStationCapabilities" => {
                nested("InstrumentCapabilities", "instrument_capabilities")?;
            }
            "StationDescriptor" if !value["capabilities"].is_null() => {
                nested("PublicStationCapabilities", "capabilities")?;
                if value["station_id"] != value["capabilities"]["station_id"]
                    || value["mode"] != value["capabilities"]["mode"]
                {
                    return Err(at
                        .fail(
                            "STATION_IDENTITY_MISMATCH",
                            "capabilities",
                            "Station descriptor and capabilities disagree",
                        )
                        .into());
                }
            }
            "OperationRecord" if !value["experiment"].is_null() => {
                nested("ExperimentRecord", "experiment")?
            }
            "InstrumentCapabilities" => nested("GeneratorCapability", "generator")?,
            "GeneratorCapability" => generator_capability(value, at)?,
            "InstrumentationExecutionPlan" => execution_plan(value, at)?,
            _ => {}
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Check<'a> {
    path: &'a str,
    profile: &'a str,
}

impl<'a> Check<'a> {
    fn fail(self, code: &str, field: &str, message: &str) -> ValidationIssue {
        ValidationIssue::new(code, &child(self.path, field), message, self.profile)
    }

    fn here(self, code: &str, message: &str) -> ValidationIssue {
        ValidationIssue::new(code, self.path, message, self.profile)
    }
}

// Semantic checks run on already schema-validated instances.
fn array(value: &Value) -> &[Value] {
    value.as_array().expect("schema-validated array")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("schema-validated string")
}

fn strings(value: &Value) -> HashSet<&str> {
    array(value).iter().map(text).collect()
}

fn greater(left: &Value, right: &Value) -> bool {
    numeric::compare(left, right) == Ordering::Greater
}

fn count_differs(count: usize, value: &Value) -> bool {
    numeric::compare_count(count, value) != Ordering::Equal
}

fn assignment_definition(value: &Value) -> &'static str {
    if value["schema_version"] == "2.0" {
        "AssignmentDefinitionV2"
    } else {
        "AssignmentDefinition"
    }
}

fn unique(entries: &[Value], key: &str, at: Check<'_>) -> Result<(), ValidationIssue> {
    let mut seen = HashSet::new();
    for (index, entry) in entries.iter().enumerate() {
        if !seen.insert(text(&entry[key])) {
            return Err(ValidationIssue::new(
                "DUPLICATE_ID",
                &child(&child(at.path, index), key),
                format!("duplicate {key}"),
                at.profile,
            ));
        }
    }
    Ok(())
}

fn endpoint(value: &Value, at: Check<'_>) -> Result<(), ValidationIssue> {
    let forms = [
        "node_id",
        "fixed_node_id",
        "reference_node_id",
        "design_port",
    ];
    if forms
        .iter()
        .filter(|key| value.get(**key).is_some())
        .count()
        > 1
    {
        return Err(at.here(
            "ENDPOINT_BINDING_CONFLICT",
            "endpoint binding forms are mutually exclusive",
        ));
    }
    Ok(())
}

fn component(value: &Value, at: Check<'_>) -> Result<(), ValidationIssue> {
    if value["kind"] == "OPAQUE_PHYSICAL_COMPONENT" && value["identity"] != "UNKNOWN" {
        return Err(at.fail(
            "OPAQUE_IDENTITY",
            "identity",
            "opaque component identity must be UNKNOWN",
        ));
    }
    if let Some(pins) = value.get("pins") {
        unique(
            array(pins),
            "pin_id",
            Check {
                path: &child(at.path, "pins"),
                ..at
            },
        )?;
    }
    Ok(())
}

fn topology(value: &Value, at: Check<'_>) -> Result<(), ValidationIssue> {
    unique(
        array(&value["nodes"]),
        "node_id",
        Check {
            path: &child(at.path, "nodes"),
            ..at
        },
    )?;
    unique(
        array(&value["components"]),
        "component_id",
        Check {
            path: &child(at.path, "components"),
            ..at
        },
    )?;
    let nodes: HashSet<_> = array(&value["nodes"])
        .iter()
        .map(|node| text(&node["node_id"]))
        .collect();
    let mut connections = HashSet::new();
    for (index, connection) in array(&value["connections"]).iter().enumerate() {
        let path = child(&child(at.path, "connections"), index);
        let location = Check { path: &path, ..at };
        let members = strings(&connection["node_ids"]);
        if members.len() != array(&connection["node_ids"]).len() || !members.is_subset(&nodes) {
            return Err(location.fail(
                "INVALID_NODE_REFERENCE",
                "node_ids",
                "connection contains duplicate or missing node",
            ));
        }
        let mut ordered: Vec<_> = members.into_iter().collect();
        ordered.sort_unstable();
        if !connections.insert(ordered) {
            return Err(location.here("DUPLICATE_CONNECTION", "duplicate electrical connection"));
        }
    }
    for (index, value) in array(&value["components"]).iter().enumerate() {
        let path = child(&child(at.path, "components"), index);
        let location = Check { path: &path, ..at };
        component(value, location)?;
        let members = strings(&value["node_ids"]);
        let pins = value["pins"].as_array().into_iter().flatten();
        if members.len() != array(&value["node_ids"]).len()
            || !members.is_subset(&nodes)
            || pins
                .into_iter()
                .any(|pin| !members.contains(text(&pin["node_id"])))
        {
            return Err(location.here(
                "INVALID_NODE_REFERENCE",
                "component or pin references a missing node",
            ));
        }
    }
    Ok(())
}

fn acknowledgement(value: &Value, at: Check<'_>) -> Result<(), ValidationIssue> {
    let status = text(&value["status"]);
    let error_code = &value["error_code"];
    let error_valid = match status {
        "FAILED" => !error_code.is_null(),
        "REQUESTED" => error_code.is_null() || error_code == "ACKNOWLEDGEMENT_UNKNOWN",
        _ => error_code.is_null(),
    };
    if (value["accepted"] == true) != matches!(status, "ACCEPTED" | "APPLIED")
        || (value["hardware_applied"] == true) != (status == "APPLIED")
        || !error_valid
        || status != "REQUESTED"
            && numeric::compare(&value["revision"], &Value::from(1)) == Ordering::Less
    {
        return Err(at.fail(
            "ACKNOWLEDGEMENT_MISMATCH",
            "status",
            "Acknowledgement status and evidence disagree",
        ));
    }
    Ok(())
}

fn capture_metadata(value: &Value, at: Check<'_>) -> Result<(), ValidationIssue> {
    let digital = value["encoding"] == "digital_events";
    if value["instrument"] == "logic_analyzer" && !digital
        || value["instrument"] == "oscilloscope" && digital
    {
        return Err(at.fail(
            "CAPTURE_ENCODING_MISMATCH",
            "encoding",
            "Capture instrument/encoding disagree",
        ));
    }
    if let Some(signals) = value["signals"].as_array() {
        let unique = strings(&value["signals"]);
        if unique.len() != signals.len() || count_differs(signals.len(), &value["channel_count"]) {
            return Err(at.fail(
                "CAPTURE_CHANNEL_MISMATCH",
                "signals",
                "Capture channels are inconsistent",
            ));
        }
    }
    let available = value["available"] == true;
    let no_payload = value["payload_ref"].is_null();
    let payload_bytes = &value["payload_byte_count"];
    if available && (no_payload || !greater(payload_bytes, &Value::from(0)))
        || !available
            && (!no_payload || count_differs(0, payload_bytes) || !value["sha256"].is_null())
    {
        return Err(at.fail(
            "CAPTURE_AVAILABILITY_MISMATCH",
            "available",
            "Capture availability is inconsistent",
        ));
    }
    if digital
        && (greater(&value["sample_count"], &Value::from(100000))
            || greater(payload_bytes, &Value::from(9 + 65536 + 100000 * 11)))
    {
        return Err(at.here(
            "CAPTURE_PAYLOAD_LIMIT",
            "Capture exceeds NCAP v1 digital limits",
        ));
    }
    Ok(())
}

fn generator_capability(value: &Value, at: Check<'_>) -> Result<(), ValidationIssue> {
    for (lower, upper) in [
        ("frequency_min_hz", "frequency_max_hz"),
        ("offset_min_v", "offset_max_v"),
    ] {
        if greater(&value[lower], &value[upper]) {
            return Err(at.fail("PROFILE_LIMIT_ORDER", lower, "Inconsistent profile limits"));
        }
    }
    if let Some(waveforms) = value["supported_waveforms"].as_array() {
        if strings(&value["supported_waveforms"]).len() != waveforms.len() {
            return Err(at.fail(
                "DUPLICATE_WAVEFORM",
                "supported_waveforms",
                "Duplicate supported waveform",
            ));
        }
    }
    Ok(())
}

fn execution_plan(value: &Value, at: Check<'_>) -> Result<(), ValidationIssue> {
    let bindings = array(&value["endpoint_bindings"]);
    unique(
        bindings,
        "endpoint_id",
        Check {
            path: &child(at.path, "endpoint_bindings"),
            ..at
        },
    )?;
    unique(
        array(&value["routing_allocations"]),
        "resource_id",
        Check {
            path: &child(at.path, "routing_allocations"),
            ..at
        },
    )?;
    let endpoints: HashSet<_> = bindings
        .iter()
        .map(|entry| text(&entry["endpoint_id"]))
        .collect();
    for key in ["routing_allocations", "instrument_allocations"] {
        for (index, entry) in array(&value[key]).iter().enumerate() {
            if !endpoints.contains(text(&entry["endpoint_id"])) {
                let path = child(&child(at.path, key), index);
                return Err(ValidationIssue::new(
                    "UNBOUND_ENDPOINT",
                    &child(&path, "endpoint_id"),
                    "allocation references an unbound endpoint",
                    at.profile,
                ));
            }
        }
    }
    Ok(())
}

/// Apply shared sampled-capture relationships to an already shape-validated document.
pub(crate) fn validate_capture_semantics(
    name: &str,
    value: &Value,
    profile: &str,
) -> Result<(), ValidationIssue> {
    numeric::check_finite(value, "", profile)?;
    analog_capture(name, value, Check { path: "", profile })
}

fn analog_capture(name: &str, value: &Value, at: Check<'_>) -> Result<(), ValidationIssue> {
    match name {
        "AnalogCaptureDescriptor" => analog_capture(
            "AnalogCaptureHeader",
            &value["header"],
            Check {
                path: &child(at.path, "header"),
                ..at
            },
        )?,
        "AnalogCaptureChannel" => {
            if numeric::compare(&value["range_min"], &value["range_max"]) != Ordering::Less {
                return Err(at.fail(
                    "CAPTURE_RANGE_ORDER",
                    "range_max",
                    "Capture channel range must increase",
                ));
            }
        }
        "AnalogCaptureTimeAxis" => {
            if value["clock_id"].is_null() == (value["reference"] == "device_clock") {
                return Err(at.fail(
                    "CAPTURE_CLOCK_REFERENCE",
                    "clock_id",
                    "Only a device clock requires a clock identity",
                ));
            }
            if value["reference"] == "capture_relative" && value["anchor_ns"] != "0" {
                return Err(at.fail(
                    "CAPTURE_TIME_ANCHOR",
                    "anchor_ns",
                    "Capture-relative time starts at zero",
                ));
            }
        }
        "AnalogCaptureQuality" => capture_quality(value, at, None, None)?,
        "AnalogCaptureHeader" => {
            let mut seen = HashSet::new();
            for (index, channel) in array(&value["channels"]).iter().enumerate() {
                let path = child(&child(at.path, "channels"), index);
                let location = Check { path: &path, ..at };
                if !seen.insert(text(&channel["id"])) {
                    return Err(location.fail("DUPLICATE_ID", "id", "Duplicate capture channel"));
                }
                analog_capture("AnalogCaptureChannel", channel, location)?;
            }
            if value["provenance"] == "synthetic_fixture" && value["source"] != "simulation" {
                return Err(at.fail(
                    "CAPTURE_PROVENANCE_MISMATCH",
                    "provenance",
                    "Synthetic fixture must be labelled simulation",
                ));
            }
            analog_capture(
                "AnalogCaptureTimeAxis",
                &value["time_axis"],
                Check {
                    path: &child(at.path, "time_axis"),
                    ..at
                },
            )?;
            capture_quality(
                &value["quality"],
                Check {
                    path: &child(at.path, "quality"),
                    ..at
                },
                Some(numeric::exact_integer(&value["frame_count"])),
                Some(&seen),
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn capture_quality(
    value: &Value,
    at: Check<'_>,
    count: Option<BigInt>,
    ids: Option<&HashSet<&str>>,
) -> Result<(), ValidationIssue> {
    if ids.is_some_and(|ids| !strings(&value["clipped_channels"]).is_subset(ids)) {
        return Err(at.fail(
            "CAPTURE_CHANNEL_REFERENCE",
            "clipped_channels",
            "Clipped channel is absent from channel table",
        ));
    }
    let mut previous = BigInt::from(-1);
    let mut missing = BigInt::from(0);
    let lost = numeric::exact_integer(&value["lost_frames"]);
    for (index, gap) in array(&value["gaps"]).iter().enumerate() {
        let frame = numeric::exact_integer(&gap["at_frame"]);
        if frame <= previous || count.as_ref().is_some_and(|count| &frame > count) {
            let path = child(&child(at.path, "gaps"), index);
            return Err(ValidationIssue::new(
                "CAPTURE_GAP_ORDER",
                &path,
                "Gaps must be ordered within the capture",
                at.profile,
            ));
        }
        previous = frame;
        missing += numeric::exact_integer(&gap["missing_frames"]);
    }
    if missing != lost {
        return Err(at.fail(
            "CAPTURE_LOSS_MISMATCH",
            "gaps",
            "Gaps must account for every lost frame",
        ));
    }
    let ranges = array(&value["invalid_ranges"]);
    let mut end = BigInt::from(0);
    for (index, range) in ranges.iter().enumerate() {
        let first = numeric::exact_integer(&range["first_frame"]);
        let next = &first + numeric::exact_integer(&range["frame_count"]);
        if first < end || count.as_ref().is_some_and(|count| &next > count) {
            let path = child(&child(at.path, "invalid_ranges"), index);
            return Err(ValidationIssue::new(
                "CAPTURE_INVALID_RANGE",
                &path,
                "Invalid ranges must be ordered, disjoint and within stored frames",
                at.profile,
            ));
        }
        end = next;
    }
    let status = text(&value["status"]);
    let termination = text(&value["termination_reason"]);
    let has_loss = lost != BigInt::from(0);
    let inconsistent = match status {
        "VALID" => has_loss || !ranges.is_empty() || termination != "COMPLETE",
        "PARTIAL" => has_loss || termination == "COMPLETE" && ranges.is_empty(),
        "LOSS" => !has_loss,
        _ => has_loss && status != "INVALID",
    };
    if inconsistent || termination == "TRANSPORT_LOSS" && !has_loss && status != "INVALID" {
        return Err(at.fail(
            "CAPTURE_QUALITY_MISMATCH",
            "status",
            "Capture quality and termination evidence disagree",
        ));
    }
    Ok(())
}
