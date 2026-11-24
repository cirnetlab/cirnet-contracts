"""Verify the opt-in Rust binding against frozen Contracts 0.1.2 outcomes."""
from functools import partial
from importlib import util
import json
import os
from pathlib import Path

import pytest

from google.protobuf.descriptor_pb2 import FileDescriptorProto, FileDescriptorSet
from google.protobuf.text_format import Parse


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests/fixtures"
REFERENCE = json.loads((FIXTURES / "contracts-0.1.2-parity.json").read_text(encoding="utf-8"))


def read_cases(relative):
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))["cases"]


def reference_cases(relative, group):
    cases = read_cases(relative)
    expected = REFERENCE[group]
    assert len(cases) == len(expected)
    assert {case["id"] for case in cases} == expected.keys()
    return cases


@pytest.fixture(scope="module")
def native():
    extension = Path(os.environ["CIRNET_CONTRACTS_NATIVE_BINDING"]).resolve(strict=True)
    spec = util.spec_from_file_location("cirnet_contracts_native", extension)
    module = util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@pytest.fixture(scope="module")
def registry(native):
    return native.Contracts(str(ROOT))


def validation_error(registry, name, profile, payload):
    with pytest.raises(ValueError) as failure:
        registry.validate(name, profile, payload)
    return json.loads(str(failure.value))


@pytest.mark.parametrize(
    "case", reference_cases("contracts/fixtures/conformance.json", "validation"),
    ids=lambda case: case["id"],
)
def test_validation_matches_reference(registry, case):
    name, profile, value = case["contract"], case["profile"], case["value"]
    payload = json.dumps(value)
    expected = REFERENCE["validation"][case["id"]]
    if case["valid"]:
        assert json.loads(registry.validate(name, profile, payload)) == expected["value"]
    else:
        issue = expected["issue"]
        actual = validation_error(registry, name, profile, payload)
        assert {key: actual[key] for key in issue} == issue


def test_raw_object_cannot_bypass_type_validation(registry):
    payload = '{"signal":"out","time_ns":{"$serde_json::private::Number":"1"},"value":0}'
    issue = validation_error(registry, "DigitalObservation", "transport", payload)
    assert (issue["code"], issue["path"]) == ("SCHEMA_TYPE", "/time_ns")


@pytest.mark.parametrize("token", ("1.0000000000000000001", "1e-400", "1.01e-9223372036854775807"))
def test_binary64_underflow_and_rounding_match_reference(registry, token):
    payload = '{"signal":"out","time_ns":' + token + ',"value":0}'
    observed = json.loads(registry.validate("DigitalObservation", "transport", payload))
    assert observed == REFERENCE["binary64"][token]["value"]


def test_rounding_outside_safe_integer_limit_matches_reference(registry):
    token = "9007199254740991.5"
    payload = '{"signal":"out","time_ns":' + token + ',"value":0}'
    issue = validation_error(registry, "DigitalObservation", "transport", payload)
    expected = REFERENCE["binary64"][token]["issue"]
    assert {key: issue[key] for key in expected} == expected


@pytest.mark.parametrize(
    "case", reference_cases("contracts/fixtures/capture/conformance.json", "capture"),
    ids=lambda case: case["id"],
)
def test_ncap_matches_reference(native, case):
    payload = bytes.fromhex(case["payload_hex"])
    if case["version"] == 1:
        decode = partial(native.decode_digital, payload)
    else:
        decode = partial(native.decode_sampled, payload, json.dumps(case["descriptor"]))

    expected = REFERENCE["capture"][case["id"]]
    if not case["valid"]:
        with pytest.raises(ValueError) as failure:
            decode()
        assert str(failure.value) == expected["error"]
        return

    decoded = expected["decoded"]
    assert json.loads(decode()) == decoded
    if case.get("canonical_encoding"):
        if case["version"] == 1:
            encoded = native.encode_digital(
                json.dumps(decoded["samples"]), json.dumps(decoded["duration_ns"])
            )
        else:
            encoded = native.encode_sampled(
                json.dumps(decoded["descriptor"]["header"]), json.dumps(decoded["samples"])
            )
        assert encoded == payload
    if case["version"] == 2:
        frame_times = expected["frameTimes"]
        assert len(frame_times) == len(case["expected"]["times_ns"])
        for index, expected_time in enumerate(frame_times):
            assert native.sampled_frame_time(json.dumps(decoded), str(index)) == tuple(expected_time)


def test_descriptor_matches_reference(native):
    canonical = FileDescriptorSet.FromString(native.hardware_descriptor(False)).file[0]
    legacy = FileDescriptorSet.FromString(native.hardware_descriptor(True)).file[0]
    reference = Parse(
        (FIXTURES / "contracts-0.1.2-rpc-v2.textproto").read_text(encoding="utf-8"),
        FileDescriptorProto(),
    )

    assert canonical == reference
    assert canonical.package == "cirnet.hardware.v2"
    assert legacy.package == "netcircuit.hardware.v2"
    assert [method.name for method in canonical.service[0].method] == [
        method.name for method in legacy.service[0].method
    ]


@pytest.mark.parametrize("name", ("LabEndpoint", "TestVector", "LabelPropertiesV1"))
def test_terminal_line_feed_in_plain_patterns_matches_reference(registry, name):
    expected = REFERENCE["terminalLineFeed"][name]
    assert json.loads(registry.validate(name, "transport", json.dumps(expected["input"]))) == expected["value"]


@pytest.mark.parametrize(
    ("fixture", "case_id"),
    [
        ("flexible-mounting-parity.json", "axial-horizontal"),
        ("sealed-package-parity.json", "sealed-generic_resistor-11"),
    ],
)
def test_flexible_world_pose_overflow_matches_reference(registry, fixture, case_id):
    cases = read_cases("contracts/design-models/fixtures/" + fixture)
    graph = next(case["graph"] for case in cases if case["id"] == case_id)
    board = next(module for module in graph["modules"] if module["type"].startswith("BREADBOARD"))
    board.setdefault("position", {})["x"] = 10**400
    value = {
        "schema_version": "2.0", "assignment_id": "reference", "title": "Reference",
        "lab_endpoints": [], "assessment_policy": "FUNCTIONAL_ONLY", "reference_design_graph": graph,
    }
    actual = validation_error(registry, "AssignmentDefinitionV2", "transport", json.dumps(value))
    assert actual == REFERENCE["flexibleWorldPoseOverflow"][case_id]


@pytest.mark.parametrize("profile", ["structural", "transport"])
@pytest.mark.parametrize(
    ("key", "code", "path"),
    [
        ("endpoint_id", "SCHEMA_PATTERN", "/endpoint_id"),
        ("channel", "SCHEMA_MINIMUM", "/channel"),
    ],
)
def test_endpoint_first_issue_matches_reference(registry, profile, key, code, path):
    expected = REFERENCE["endpointFirstIssue"][profile][key]
    actual = validation_error(registry, "LabEndpoint", profile, json.dumps(expected["input"]))
    assert (actual["code"], actual["path"]) == (expected["issue"]["code"], expected["issue"]["path"]) == (code, path)
