use cirnet_contracts_native::validation::{Registry, ValidationError};
use serde_json::{Value, json};
use std::{fs, path::Path, sync::OnceLock};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| Registry::load(root()).unwrap())
}

fn issue(result: Result<Value, ValidationError>, code: &str, path: &str, profile: &str) {
    let ValidationError::Invalid(issue) = result.unwrap_err() else {
        panic!("expected a validation issue");
    };
    assert_eq!(issue.code, code);
    assert_eq!(issue.path, path);
    assert_eq!(issue.profile, profile);
    assert_eq!(issue.severity, "error");
}

fn corpus(file: &str, default_name: &str) {
    let data: Value =
        serde_json::from_slice(&fs::read(root().join("contracts/fixtures").join(file)).unwrap())
            .unwrap();
    for case in data["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let name = case["contract"].as_str().unwrap_or(default_name);
        let profile = case["profile"].as_str().unwrap_or("transport");
        let value = &case["value"];
        let result = registry().validate(name, profile, value);
        if case["valid"] == true {
            assert_eq!(
                result.unwrap_or_else(|error| panic!("{id}: {error}")),
                *value,
                "{id}"
            );
        } else {
            let error = result.expect_err(id);
            let ValidationError::Invalid(found) = error else {
                panic!("{id}: {error}");
            };
            assert_eq!(found.profile, profile, "{id}");
            let expected = case.get("issue").unwrap_or(case);
            if let Some(code) = expected["code"].as_str() {
                assert_eq!(found.code, code, "{id}");
                assert_eq!(found.path, expected["path"].as_str().unwrap(), "{id}");
            }
        }
    }
}

#[test]
fn shared_profiles_match_conformance_corpus() {
    corpus("conformance.json", "");
}

#[test]
fn authoring_and_reference_design_match_conformance_corpus() {
    corpus("assignment-v2-conformance.json", "");
}

#[test]
fn instrument_acknowledgements_match_conformance_corpus() {
    corpus("instrument-control-conformance.json", "");
}

#[test]
fn experiment_history_matches_conformance_corpus() {
    corpus("history-conformance.json", "ExperimentHistoryPage");
}

#[test]
fn unknown_contract_and_profile_are_distinct_from_invalid_data() {
    assert!(matches!(
        registry().validate("does-not-exist", "structural", &Value::Null),
        Err(ValidationError::UnknownContract(name)) if name == "does-not-exist"
    ));
    assert!(matches!(
        registry().validate("DigitalObservation", "typo", &Value::Null),
        Err(ValidationError::UnknownProfile(name)) if name == "typo"
    ));
}

#[test]
fn required_and_semantic_paths_escape_property_names() {
    issue(
        registry().validate(
            "DigitalObservation",
            "transport",
            &json!({"time_ns":0,"value":1}),
        ),
        "SCHEMA_REQUIRED",
        "/signal",
        "transport",
    );
    let module = json!({
        "id":"CLK", "type":"CLOCK",
        "signal_domains":{"version":"1.0","ports":{"A~/B":"CLOCK"}}
    });
    issue(
        registry().validate("CircuitModule", "editor", &module),
        "UNKNOWN_SIGNAL_PORT",
        "/signal_domains/ports/A~0~1B",
        "editor",
    );
}

#[test]
fn numeric_profiles_preserve_exact_values_and_reject_overflow() {
    let huge: Value = serde_json::from_str(
        r#"{"signal":"CLK.OUT","time_ns":18446744073709551616000000000001,"value":1}"#,
    )
    .unwrap();
    assert_eq!(
        registry()
            .validate("DigitalObservation", "structural", &huge)
            .unwrap(),
        huge
    );
    issue(
        registry().validate("DigitalObservation", "editor", &huge),
        "SCHEMA_MAXIMUM",
        "/time_ns",
        "editor",
    );
    let clock = json!({"signal":"CLK.OUT","period_ns":4294967296_u64,"cycles":1});
    assert_eq!(
        registry()
            .validate("ClockConfiguration", "editor", &clock)
            .unwrap(),
        clock
    );
    issue(
        registry().validate("ClockConfiguration", "transport", &clock),
        "SCHEMA_MAXIMUM",
        "/period_ns",
        "transport",
    );
    let capture = serde_json::from_str(
        r#"{"schema_version":"1.0","capture_id":"c","source":"simulation","instrument":"logic_analyzer","encoding":"digital_events","available":true,"duration_ns":18446744073709551616,"sample_count":1,"samples":[{"signal":"out","time_ns":18446744073709551617,"value":0}]}"#,
    ).unwrap();
    issue(
        registry().validate("CaptureResult", "structural", &capture),
        "CAPTURE_TIME_OUTSIDE_DURATION",
        "/samples/0/time_ns",
        "structural",
    );
}

#[test]
fn schema_defaults_never_change_optional_presence() {
    let absent = json!({"id":"CLK","type":"CLOCK"});
    let explicit = json!({"id":"CLK","type":"CLOCK","properties":{}});
    assert_eq!(
        registry()
            .validate("CircuitModule", "editor", &absent)
            .unwrap(),
        absent
    );
    assert_eq!(
        registry()
            .validate("CircuitModule", "editor", &explicit)
            .unwrap(),
        explicit
    );
}

#[test]
fn legacy_uri_fragment_resolves_to_the_canonical_schema() {
    let canonical =
        "https://cirnet.local/contracts/physical-lab/1.0/lab.schema.json#/$defs/DigitalObservation";
    let legacy = canonical.replace("cirnet.local", "netcircuit.local");
    let schema = registry().load_schema(canonical).unwrap();
    assert_eq!(schema["properties"]["time_ns"]["type"], "integer");
    assert_eq!(registry().load_schema(&legacy).unwrap(), schema);
    assert!(
        registry()
            .load_schema("https://example.org/schema.json")
            .is_err()
    );
}

#[test]
fn bundle_relocates_relative_and_legacy_refs_without_mutating_source() {
    let bundle = registry()
        .schema_bundle(true, "transport", "#/$defs/")
        .unwrap();
    assert_eq!(
        bundle["$defs"]["CircuitGraph"]["properties"]["modules"]["items"]["$ref"],
        "#/$defs/CircuitModule"
    );
    assert_eq!(
        bundle["$defs"]["ClockConfiguration"]["properties"]["period_ns"]["maximum"],
        4294967295_u64
    );
    assert!(bundle["$defs"].get("StationCapabilities").is_none());
    let source = registry()
        .schema_bundle(false, "structural", "#/$defs/")
        .unwrap();
    assert!(
        source["$defs"]["ClockConfiguration"]["properties"]["period_ns"]
            .get("maximum")
            .is_none()
    );
}

#[test]
fn analog_capture_quality_paths_point_to_nested_ranges() {
    let value = json!({
        "format":"NCAP_SAMPLED_V2", "capture_id":"synthetic", "generation":0,
        "source":"simulation", "provenance":"synthetic_fixture", "instrument":"oscilloscope",
        "encoding":"INT16", "byte_order":"little", "layout":"frame_interleaved", "frame_count":3,
        "channels":[{"id":"CH1","unit":"V","scale":0.01,"offset":0,"range_min":-10,"range_max":10,"calibration_id":null}],
        "time_axis":{"reference":"unix_epoch","clock_id":null,"anchor_ns":"9007199254740993000","sample_rate_hz":{"numerator":3,"denominator":2}},
        "quality":{"status":"PARTIAL","termination_reason":"COMPLETE","lost_frames":0,"gaps":[],"invalid_ranges":[{"first_frame":2,"frame_count":2}],"clipped_channels":[]}
    });
    issue(
        registry().validate("AnalogCaptureHeader", "structural", &value),
        "CAPTURE_INVALID_RANGE",
        "/quality/invalid_ranges/0",
        "structural",
    );
}

#[test]
fn reference_designs_match_rigid_flexible_and_sealed_fixture_corpora() {
    for file in [
        "flexible-mounting-parity.json",
        "rigid-leads-parity.json",
        "sealed-package-parity.json",
    ] {
        let data: Value = serde_json::from_slice(
            &fs::read(root().join("contracts/design-models/fixtures").join(file)).unwrap(),
        )
        .unwrap();
        for case in data["cases"].as_array().unwrap() {
            let id = case["id"].as_str().unwrap();
            let value = json!({
                "schema_version":"2.0",
                "assignment_id":"reference",
                "title":"Reference",
                "lab_endpoints":[],
                "assessment_policy":"FUNCTIONAL_ONLY",
                "reference_design_graph":case["graph"],
            });
            let result = registry().validate("AssignmentDefinitionV2", "transport", &value);
            if case["valid"] == true {
                assert_eq!(
                    result.unwrap_or_else(|error| panic!("{id}: {error}")),
                    value,
                    "{id}"
                );
            } else {
                let ValidationError::Invalid(issue) = result.expect_err(id) else {
                    panic!("{id}: expected validation issue");
                };
                assert!(
                    issue.path.starts_with("/reference_design_graph"),
                    "{id}: {issue:?}"
                );
                if let Some(reason) = case["reason"].as_str() {
                    let (before, after) = reason.split_once(".*").unwrap();
                    assert!(
                        issue
                            .message
                            .split_once(before)
                            .is_some_and(|(_, tail)| tail.contains(after)),
                        "{id}: {issue:?}",
                    );
                }
            }
        }
    }
}

#[test]
fn nested_runtime_references_accept_aliases_and_reject_external_resolution() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    fs::create_dir(root.join("contracts")).unwrap();
    fs::copy(
        crate::root().join("contracts/profiles.json"),
        root.join("contracts/profiles.json"),
    )
    .unwrap();
    fs::write(
        root.join("contracts/manifest.json"),
        r#"{"contracts":[
            {"id":"Probe","kind":"schema","path":"contracts/probe.json","uri":"https://cirnet.local/a/probe.json","pointer":"","visibility":"public"},
            {"id":"Integer","kind":"schema","path":"contracts/leaf.json","uri":"https://cirnet.local/a/leaf.json","pointer":"/$defs/Integer","visibility":"public"}
        ]}"#,
    ).unwrap();
    fs::write(
        root.join("contracts/leaf.json"),
        r##"{"$defs":{"Sibling":{"type":"integer"},"Integer":{"$ref":"#/$defs/Sibling"}}}"##,
    )
    .unwrap();
    let write_probe = |reference| {
        let probe = json!({"$ref":reference});
        fs::write(root.join("contracts/probe.json"), probe.to_string()).unwrap();
    };
    write_probe("https://netcircuit.local/a/leaf.json#/$defs/Integer");
    let registry = Registry::load(root).unwrap();
    assert_eq!(
        registry.validate("Probe", "editor", &json!(3)).unwrap(),
        json!(3)
    );
    issue(
        registry.validate("Probe", "editor", &json!("3")),
        "SCHEMA_TYPE",
        "",
        "editor",
    );
    write_probe("https://example.invalid/a/leaf.json");
    assert!(matches!(
        Registry::load(root)
            .unwrap()
            .validate("Probe", "structural", &json!(3)),
        Err(ValidationError::Registry(_))
    ));
}

#[test]
fn legacy_float_tokens_round_for_integer_validation_and_return_the_parsed_value() {
    for (token, rounded) in [
        ("1.0000000000000000001", "1.0"),
        ("1e-400", "0.0"),
        ("-1e-400", "-0.0"),
    ] {
        let raw = format!(r#"{{"signal":"x","time_ns":{token},"value":0}}"#);
        let value: Value = serde_json::from_str(&raw).unwrap();
        let result = registry()
            .validate("DigitalObservation", "transport", &value)
            .unwrap_or_else(|error| panic!("{token}: {error}"));
        assert_eq!(
            result.to_string(),
            format!(r#"{{"signal":"x","time_ns":{rounded},"value":0}}"#),
        );
    }

    let value: Value =
        serde_json::from_str(r#"{"signal":"x","time_ns":9007199254740991.5,"value":0}"#).unwrap();
    issue(
        registry().validate("DigitalObservation", "transport", &value),
        "SCHEMA_MAXIMUM",
        "/time_ns",
        "transport",
    );
}

#[test]
fn exponent_underflow_is_bounded_before_evaluating_the_schema() {
    let value: Value =
        serde_json::from_str(r#"{"signal":"x","time_ns":1.01e-9223372036854775807,"value":0}"#)
            .unwrap();
    assert_eq!(
        registry()
            .validate("DigitalObservation", "transport", &value)
            .unwrap(),
        json!({"signal":"x","time_ns":0.0,"value":0}),
    );
}

#[cfg(unix)]
#[test]
fn manifest_rejects_symlinked_schemas_outside_the_source_root() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("source");
    let contracts = root.join("contracts");
    fs::create_dir_all(&contracts).unwrap();
    fs::copy(
        crate::root().join("contracts/profiles.json"),
        contracts.join("profiles.json"),
    )
    .unwrap();
    fs::write(
        contracts.join("manifest.json"),
        r#"{"contracts":[
            {"id":"Probe","kind":"schema","path":"contracts/probe.json","uri":"https://cirnet.local/probe.json","pointer":"","visibility":"public"}
        ]}"#,
    ).unwrap();
    let external = temporary.path().join("external.json");
    fs::write(&external, r#"{"type":"integer"}"#).unwrap();
    std::os::unix::fs::symlink(&external, contracts.join("probe.json")).unwrap();
    let Err(error) = Registry::load(&root) else {
        panic!("external symlink escaped the source root");
    };
    assert!(
        error.to_string().contains("outside the source root"),
        "{error}"
    );
}

#[test]
fn overflowed_decimal_reports_the_escaped_nonfinite_path() {
    let value: Value = serde_json::from_str(
        r#"{"id":"CLK","type":"CLOCK","properties":{"a~/b":1e9999999999999999999999}}"#,
    )
    .unwrap();
    issue(
        registry().validate("CircuitModule", "structural", &value),
        "NONFINITE_NUMBER",
        "/properties/a~0~1b",
        "structural",
    );
}

#[test]
fn plain_runtime_patterns_accept_one_trailing_line_feed() {
    for (name, value) in [
        (
            "LabEndpoint",
            json!({"endpoint_id":"input\n","role":"INPUT_SOURCE"}),
        ),
        ("TestVector", json!({"time_ns":0,"inputs":{"input\n":1}})),
        (
            "LabelPropertiesV1",
            json!({
                "version":1,"text":"Hello","fontSizeMm":3,
                "textColor":"#20252A\n","backgroundColor":"#FAFAF7","paddingMm":1
            }),
        ),
    ] {
        assert_eq!(
            registry().validate(name, "transport", &value).unwrap(),
            value,
            "{name}"
        );
    }
}

#[test]
fn strict_end_patterns_still_reject_trailing_line_feed() {
    issue(
        registry().validate(
            "LabelPropertiesV1",
            "transport",
            &json!({
                "version":1,"text":"one\ntwo\nthree\nfour\n","fontSizeMm":3,
                "textColor":"#20252A","backgroundColor":"#FAFAF7","paddingMm":1
            }),
        ),
        "SCHEMA_PATTERN",
        "/text",
        "transport",
    );
}

#[test]
fn sdk_patterns_retain_the_canonical_end_anchors() {
    let bundle = registry()
        .schema_bundle(true, "transport", "#/$defs/")
        .unwrap();
    assert_eq!(
        bundle["$defs"]["LabEndpoint"]["properties"]["endpoint_id"]["pattern"],
        "^[A-Za-z0-9_][A-Za-z0-9_.:-]*$",
    );
}

#[test]
fn pattern_properties_adapt_end_anchors_without_rewriting_escaped_dollars() {
    let temporary = tempfile::tempdir().unwrap();
    let contracts = temporary.path().join("contracts");
    fs::create_dir(&contracts).unwrap();
    fs::copy(
        root().join("contracts/profiles.json"),
        contracts.join("profiles.json"),
    )
    .unwrap();
    fs::write(
        contracts.join("manifest.json"),
        r#"{"contracts":[
            {"id":"Probe","kind":"schema","path":"contracts/probe.json","uri":"https://cirnet.local/probe.json","pointer":"","visibility":"public"}
        ]}"#,
    ).unwrap();
    fs::write(
        contracts.join("probe.json"),
        r#"{"type":"object","patternProperties":{"^x$":{"type":"integer"},"^usd\\$":{"type":"boolean"}},"additionalProperties":false}"#,
    ).unwrap();
    let registry = Registry::load(temporary.path()).unwrap();
    let accepted = json!({"x\n":42,"usd$":true});
    assert_eq!(
        registry.validate("Probe", "structural", &accepted).unwrap(),
        accepted
    );
    assert!(
        registry
            .validate("Probe", "structural", &json!({"x\n\n":42}))
            .is_err()
    );
    assert!(
        registry
            .validate("Probe", "structural", &json!({"usd":true}))
            .is_err()
    );
    assert!(
        registry
            .validate("Probe", "structural", &json!({"x\r\n":42}))
            .is_err()
    );
}

fn reference_assignment(fixture: &str, id: &str) -> Value {
    let fixtures: Value = serde_json::from_slice(
        &fs::read(
            root()
                .join("contracts/design-models/fixtures")
                .join(fixture),
        )
        .unwrap(),
    )
    .unwrap();
    let graph = &fixtures["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()["graph"];
    json!({
        "schema_version":"2.0","assignment_id":"reference","title":"Reference",
        "lab_endpoints":[],"assessment_policy":"FUNCTIONAL_ONLY","reference_design_graph":graph,
    })
}

#[test]
fn flexible_reference_design_rejects_out_of_range_world_pose() {
    let huge: Value = serde_json::from_str(&format!("1{}", "0".repeat(400))).unwrap();
    for (fixture, id) in [
        ("flexible-mounting-parity.json", "axial-horizontal"),
        ("sealed-package-parity.json", "sealed-generic_resistor-11"),
    ] {
        for field in ["position", "rotation"] {
            let mut value = reference_assignment(fixture, id);
            let board = &mut value["reference_design_graph"]["modules"][0];
            assert!(board["type"].as_str().unwrap().starts_with("BREADBOARD"));
            if field == "position" {
                board["position"]["x"] = huge.clone();
            } else {
                board["rotation"] = huge.clone();
            }
            let ValidationError::Invalid(issue) = registry()
                .validate("AssignmentDefinitionV2", "transport", &value)
                .expect_err(id)
            else {
                panic!("{id}: expected validation issue");
            };
            assert_eq!(issue.code, "INVALID_REFERENCE_DESIGN", "{id} {field}");
            assert_eq!(issue.path, "/reference_design_graph", "{id} {field}");
            assert_eq!(
                issue.message, "Physical world pose is outside supported numerical bounds",
                "{id} {field}"
            );
        }
    }
}

#[test]
fn rigid_reference_design_wraps_integer_yaw_before_float_conversion() {
    let fixture = "rigid-leads-parity.json";
    let id = "clear-current-dip";
    let mut value = reference_assignment(fixture, id);
    let modules = value["reference_design_graph"]["modules"]
        .as_array_mut()
        .unwrap();
    let mounted = modules
        .iter_mut()
        .find(|module| module.get("mount").is_some())
        .unwrap();
    mounted["mount"]["pose"]["rotation"] = json!(3_600_000_000_000_000_360_u64);
    assert_eq!(
        registry()
            .validate("AssignmentDefinitionV2", "transport", &value)
            .unwrap(),
        value
    );
    let mut value = reference_assignment(fixture, id);
    let modules = value["reference_design_graph"]["modules"]
        .as_array_mut()
        .unwrap();
    let board = modules
        .iter_mut()
        .find(|module| module["type"].as_str().unwrap().starts_with("BREADBOARD"))
        .unwrap();
    board["rotation"] = serde_json::from_str(&format!("1{}", "0".repeat(400))).unwrap();
    assert_eq!(
        registry()
            .validate("AssignmentDefinitionV2", "transport", &value)
            .unwrap(),
        value
    );
}

#[test]
fn present_endpoint_property_failure_precedes_later_required_failure() {
    for profile in ["structural", "transport"] {
        for (value, code, path) in [
            (json!({"endpoint_id":"!"}), "SCHEMA_PATTERN", "/endpoint_id"),
            (json!({"channel":0}), "SCHEMA_MINIMUM", "/channel"),
        ] {
            issue(
                registry().validate("LabEndpoint", profile, &value),
                code,
                path,
                profile,
            );
        }
    }
}
