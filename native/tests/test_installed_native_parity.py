"""Compare the opt-in Rust test binding with the sealed Python release."""
from functools import partial
from importlib import metadata, util
import json
import os
from pathlib import Path
import sys

import pytest

from contracts import capture_codec, validation
from contracts.generated import rpc_v2_pb2
from contracts.semantic_profiles import validation_issues
from google.protobuf.descriptor_pb2 import FileDescriptorProto, FileDescriptorSet
from jsonschema import ValidationError


ROOT = Path(__file__).resolve().parents[2]


def read_cases(relative):
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))["cases"]


@pytest.fixture(scope="module")
def native():
    from contracts import __file__ as installed_contracts

    assert Path(installed_contracts).resolve().is_relative_to(Path(sys.prefix).resolve())
    provenance = json.loads(metadata.distribution("cirnet-contracts").read_text("direct_url.json"))
    assert provenance["archive_info"]["hashes"]["sha256"] == os.environ["CIRNET_CONTRACTS_PYTHON_SHA256"]
    extension = Path(os.environ["CIRNET_CONTRACTS_NATIVE_BINDING"]).resolve(strict=True)
    spec = util.spec_from_file_location("cirnet_contracts_native", extension)
    module = util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@pytest.fixture(scope="module")
def registry(native):
    return native.Contracts(str(ROOT))


@pytest.mark.parametrize("case", read_cases("contracts/fixtures/conformance.json"), ids=lambda case: case["id"])
def test_validation_matches_installed_wheel(registry, case):
    name, profile, value = case["contract"], case["profile"], case["value"]
    if case["valid"]:
        legacy = validation.validate(name, value, profile=profile)
        assert json.loads(registry.validate(name, profile, json.dumps(value))) == legacy
    else:
        with pytest.raises(ValidationError) as legacy:
            validation.validate(name, value, profile=profile)
        with pytest.raises(ValueError) as native:
            registry.validate(name, profile, json.dumps(value))
        expected = validation_issues(legacy.value, profile=profile)[0]
        actual = json.loads(str(native.value))
        keys = ("code", "path", "profile", "severity")
        assert {key: actual[key] for key in keys} == {key: expected[key] for key in keys}


def test_raw_object_cannot_bypass_type_validation(registry):
    payload = '{"signal":"out","time_ns":{"$serde_json::private::Number":"1"},"value":0}'
    with pytest.raises(ValueError) as failure:
        registry.validate("DigitalObservation", "transport", payload)
    issue = json.loads(str(failure.value))
    assert (issue["code"], issue["path"]) == ("SCHEMA_TYPE", "/time_ns")


@pytest.mark.parametrize("token", ("1.0000000000000000001", "1e-400", "1.01e-9223372036854775807"))
def test_binary64_underflow_and_rounding_match_legacy(registry, token):
    payload = '{"signal":"out","time_ns":' + token + ',"value":0}'
    expected = validation.validate("DigitalObservation", json.loads(payload), profile="transport")
    observed = json.loads(registry.validate("DigitalObservation", "transport", payload))
    assert observed == expected


def test_rounding_outside_safe_integer_limit_matches_legacy(registry):
    payload = '{"signal":"out","time_ns":9007199254740991.5,"value":0}'
    with pytest.raises(ValidationError) as legacy:
        validation.validate("DigitalObservation", json.loads(payload), profile="transport")
    with pytest.raises(ValueError) as native:
        registry.validate("DigitalObservation", "transport", payload)
    expected = validation_issues(legacy.value, profile="transport")[0]
    issue = json.loads(str(native.value))
    assert (issue["code"], issue["path"]) == (expected["code"], expected["path"])


@pytest.mark.parametrize(
    "case", read_cases("contracts/fixtures/capture/conformance.json"), ids=lambda case: case["id"]
)
def test_ncap_matches_installed_wheel(native, case):
    payload = bytes.fromhex(case["payload_hex"])
    if case["version"] == 1:
        legacy = partial(capture_codec.decode_digital, payload)
        decode = partial(native.decode_digital, payload)
    else:
        descriptor = case["descriptor"]
        legacy = partial(capture_codec.decode_sampled, payload, descriptor)
        decode = partial(native.decode_sampled, payload, json.dumps(descriptor))

    if not case["valid"]:
        with pytest.raises(ValueError) as old:
            legacy()
        with pytest.raises(ValueError) as new:
            decode()
        assert str(new.value) == str(old.value)
        return

    expected = legacy()
    assert json.loads(decode()) == expected
    if case.get("canonical_encoding"):
        if case["version"] == 1:
            encoded = native.encode_digital(
                json.dumps(expected["samples"]), json.dumps(expected["duration_ns"])
            )
        else:
            encoded = native.encode_sampled(
                json.dumps(expected["descriptor"]["header"]), json.dumps(expected["samples"])
            )
        assert encoded == payload
    if case["version"] == 2:
        for index in range(len(case["expected"]["times_ns"])):
            numerator, denominator = capture_codec.sampled_frame_time(expected, index)
            assert native.sampled_frame_time(json.dumps(expected), str(index)) == (str(numerator), denominator)


def fill_implicit_json_names(message, live):
    for field in message.field:
        if not field.HasField("json_name"):
            field.json_name = live.fields_by_name[field.name].json_name
    for nested in message.nested_type:
        fill_implicit_json_names(nested, live.nested_types_by_name[nested.name])


def test_descriptor_matches_installed_wheel(native):
    canonical = FileDescriptorSet.FromString(native.hardware_descriptor(False)).file[0]
    legacy = FileDescriptorSet.FromString(native.hardware_descriptor(True)).file[0]
    reference = FileDescriptorProto.FromString(rpc_v2_pb2.DESCRIPTOR.serialized_pb)

    # Python embeds a compact descriptor and computes default JSON names on load.
    for message in reference.message_type:
        fill_implicit_json_names(message, rpc_v2_pb2.DESCRIPTOR.message_types_by_name[message.name])
    assert canonical == reference
    assert canonical.package == "cirnet.hardware.v2"
    assert legacy.package == "netcircuit.hardware.v2"
    assert [method.name for method in canonical.service[0].method] == [
        method.name for method in legacy.service[0].method
    ]


@pytest.mark.parametrize(
    ("name", "value"),
    [
        ("LabEndpoint", {"endpoint_id": "input\n", "role": "INPUT_SOURCE"}),
        ("TestVector", {"time_ns": 0, "inputs": {"input\n": 1}}),
        ("LabelPropertiesV1", {
            "version": 1, "text": "Hello", "fontSizeMm": 3,
            "textColor": "#20252A\n", "backgroundColor": "#FAFAF7", "paddingMm": 1,
        }),
    ],
)
def test_terminal_line_feed_in_plain_patterns_matches_legacy(registry, name, value):
    expected = validation.validate(name, value, profile="transport")
    assert json.loads(registry.validate(name, "transport", json.dumps(value))) == expected


@pytest.mark.parametrize(
    ("fixture", "case_id"),
    [
        ("flexible-mounting-parity.json", "axial-horizontal"),
        ("sealed-package-parity.json", "sealed-generic_resistor-11"),
    ],
)
def test_flexible_world_pose_overflow_matches_legacy(registry, fixture, case_id):
    cases = read_cases("contracts/design-models/fixtures/" + fixture)
    graph = next(case["graph"] for case in cases if case["id"] == case_id)
    board = next(module for module in graph["modules"] if module["type"].startswith("BREADBOARD"))
    board.setdefault("position", {})["x"] = 10**400
    name, profile = "AssignmentDefinitionV2", "transport"
    value = {
        "schema_version": "2.0", "assignment_id": "reference", "title": "Reference",
        "lab_endpoints": [], "assessment_policy": "FUNCTIONAL_ONLY", "reference_design_graph": graph,
    }
    with pytest.raises(ValidationError) as legacy:
        validation.validate(name, value, profile=profile)
    with pytest.raises(ValueError) as native:
        registry.validate(name, profile, json.dumps(value))
    expected = validation_issues(legacy.value, profile=profile)[0]
    assert json.loads(str(native.value)) == expected



@pytest.mark.parametrize("profile", ["structural", "transport"])
@pytest.mark.parametrize(
    ("value", "code", "path"),
    [
        ({"endpoint_id": "!"}, "SCHEMA_PATTERN", "/endpoint_id"),
        ({"channel": 0}, "SCHEMA_MINIMUM", "/channel"),
    ],
)
def test_endpoint_first_issue_matches_legacy(registry, profile, value, code, path):
    with pytest.raises(ValidationError) as legacy:
        validation.validate("LabEndpoint", value, profile=profile)
    with pytest.raises(ValueError) as native:
        registry.validate("LabEndpoint", profile, json.dumps(value))
    expected = validation_issues(legacy.value, profile=profile)[0]
    actual = json.loads(str(native.value))
    assert (actual["code"], actual["path"]) == (expected["code"], expected["path"]) == (code, path)
