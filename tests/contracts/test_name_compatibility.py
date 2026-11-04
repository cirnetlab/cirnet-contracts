"""Canonical CIRNET names preserve supported legacy configuration and wires."""
from copy import deepcopy
import importlib
import json
from pathlib import Path
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

import pytest

from contracts.runtime_config import export_environment, getenv, simulation_worker_count
from contracts.registry import entries, load_schema, structural_validator, validate_contract

ROOT = Path(__file__).resolve().parents[2]


def test_configuration_aliases_are_synchronized_without_mutating_the_input(monkeypatch):
    original = {'NETCIRCUIT_ONLY': 'old', 'CIRNET_BOTH': '', 'NETCIRCUIT_BOTH': 'old', 'OTHER': 'keep'}
    expected = deepcopy(original)
    assert getenv('CIRNET_ONLY', environ=original) == 'old'
    assert getenv('CIRNET_BOTH', 'default', environ=original) == ''
    assert getenv('NETCIRCUIT_BOTH', environ=original) == ''
    assert getenv('MISSING', 'default', environ=original) == 'default'
    exported = export_environment(original)
    assert exported['CIRNET_ONLY'] == exported['NETCIRCUIT_ONLY'] == 'old'
    assert exported['CIRNET_BOTH'] == exported['NETCIRCUIT_BOTH'] == ''
    assert original == expected
    monkeypatch.delenv('CIRNET_SIMULATION_WORKERS', raising=False)
    monkeypatch.setenv('NETCIRCUIT_SIMULATION_WORKERS', '3')
    assert simulation_worker_count() == 3
    monkeypatch.setenv('CIRNET_SIMULATION_WORKERS', '2')
    assert simulation_worker_count() == 2
    monkeypatch.setenv('CIRNET_SIMULATION_WORKERS', '')
    with pytest.raises(ValueError): simulation_worker_count()


def test_every_old_schema_identity_resolves_offline_with_the_same_constraints():
    for entry in entries():
        if entry['kind'] != 'schema': continue
        canonical = entry['uri'] + ('#' + entry['pointer'] if entry['pointer'] else '')
        legacy = canonical.replace('https://cirnet.local/', 'https://netcircuit.local/', 1)
        assert load_schema(legacy) == load_schema(canonical)
    schema = {'$ref': 'https://netcircuit.local/contracts/public/api/1.0/schema.json#/$defs/HealthResponse'}
    for name in ('cirnet-api', 'netcircuit-api'):
        value = {'status': 'ok', 'service': name, 'hardware_mode': 'simulation'}
        structural_validator(schema).validate(value)
        assert validate_contract('HealthResponse', value) == value
    with pytest.raises(Exception):
        structural_validator(schema).validate({'status': 'ok', 'service': 'unknown', 'hardware_mode': 'simulation'})
    for name in ('cirnet-api', 'netcircuit-api'):
        hello = {'type': 'hello', 'protocol_version': '1.0', 'stream_id': 'test',
                 'sequence': 0, 'service': name, 'resync_required': False,
                 'supported_protocol_versions': ['1.0'], 'capabilities': {
                     'snapshot_resync': True, 'ordered_stream': True, 'replay_commands': False}}
        assert validate_contract('HelloEvent', hello) == hello



def test_metadata_aliases_cannot_hide_changed_contracts():
    from contracts.public_compatibility import CompatibilityError, check_compatibility
    api = json.loads((ROOT/'contracts/api/openapi.json').read_text(encoding='utf-8'))
    layout = json.loads((ROOT/'contracts/capture/layout.json').read_text(encoding='utf-8'))
    baseline = json.loads((ROOT/'contracts/compatibility/2026-10-16/public-wire-baseline.json').read_text(encoding='utf-8'))
    check_compatibility(api, layout, baseline)
    api['components']['schemas']['HealthResponse']['x-cirnet-contract']['version'] = 'unexpected'
    with pytest.raises(CompatibilityError): check_compatibility(api, layout, baseline)


def rpc_modules(tmp_path, monkeypatch):
    from contracts.codegen.generate_hardware_rpc import generate
    output = tmp_path/'compat_rpc'; generate(output)
    monkeypatch.syspath_prepend(str(tmp_path))
    return importlib.import_module('compat_rpc.rpc_v2_pb2'), importlib.import_module('compat_rpc.rpc_v2_pb2_grpc')


def test_generated_rpc_preserves_legacy_clients_and_legacy_server_routes(tmp_path, monkeypatch):
    import grpc
    pb, rpc = rpc_modules(tmp_path, monkeypatch)
    assert pb.DESCRIPTOR.package == 'cirnet.hardware.v2'
    seen = []
    class Service(rpc.HardwareServiceServicer):
        def GetCapabilities(self, request, context):
            seen.append(request.station_id)
            return pb.StationCapabilities(station_id=request.station_id)
    server = grpc.server(ThreadPoolExecutor(max_workers=2))
    rpc.add_HardwareServiceServicer_to_server(Service(), server)
    port = server.add_insecure_port('127.0.0.1:0'); server.start()
    try:
        with grpc.insecure_channel(f'127.0.0.1:{port}') as channel:
            assert rpc.HardwareServiceStub(channel).GetCapabilities(pb.Target(station_id='new'), timeout=5).station_id == 'new'
            old = channel.unary_unary('/netcircuit.hardware.v2.HardwareService/GetCapabilities',
                request_serializer=pb.Target.SerializeToString, response_deserializer=pb.StationCapabilities.FromString)
            assert old(pb.Target(station_id='old'), timeout=5).station_id == 'old'
    finally: server.stop(0).wait()
    assert seen == ['new', 'old']
    # A previous deployment only knows the old method path. New stubs retry that
    # route when the server reports UNIMPLEMENTED, preserving message bytes.
    server = grpc.server(ThreadPoolExecutor(max_workers=2))
    handlers = {'GetCapabilities': grpc.unary_unary_rpc_method_handler(
        lambda request, context: pb.StationCapabilities(station_id=request.station_id),
        request_deserializer=pb.Target.FromString, response_serializer=pb.StationCapabilities.SerializeToString)}
    server.add_generic_rpc_handlers((grpc.method_handlers_generic_handler('netcircuit.hardware.v2.HardwareService', handlers),))
    port = server.add_insecure_port('127.0.0.1:0'); server.start()
    try:
        with grpc.insecure_channel(f'127.0.0.1:{port}') as channel:
            assert rpc.HardwareServiceStub(channel).GetCapabilities(pb.Target(station_id='fallback'), timeout=5).station_id == 'fallback'
            method = rpc.HardwareServiceStub(channel).GetCapabilities
            value, call = method.with_call(pb.Target(station_id='with-call'), timeout=5)
            assert value.station_id == 'with-call' and call.code() == grpc.StatusCode.OK
            future = method.future(pb.Target(station_id='future'), timeout=5)
            assert future.result(timeout=5).station_id == 'future'
            completed = []
            future.add_done_callback(lambda result: completed.append(result.result().station_id))
            assert completed == ['future']

    finally: server.stop(0).wait()


def test_rpc_does_not_retry_a_real_server_failure(tmp_path, monkeypatch):
    import grpc
    pb, rpc = rpc_modules(tmp_path, monkeypatch)
    calls = []
    def fail(request, context):
        calls.append('canonical')
        context.abort(grpc.StatusCode.PERMISSION_DENIED, 'denied')
    server = grpc.server(ThreadPoolExecutor(max_workers=2))
    handler = grpc.unary_unary_rpc_method_handler(fail, request_deserializer=pb.Target.FromString,
                                                 response_serializer=pb.StationCapabilities.SerializeToString)
    server.add_generic_rpc_handlers((grpc.method_handlers_generic_handler('cirnet.hardware.v2.HardwareService', {'GetCapabilities': handler}),))
    port = server.add_insecure_port('127.0.0.1:0'); server.start()
    try:
        with grpc.insecure_channel(f'127.0.0.1:{port}') as channel:
            with pytest.raises(grpc.RpcError) as error:
                rpc.HardwareServiceStub(channel).GetCapabilities(pb.Target(), timeout=5)
            assert error.value.code() == grpc.StatusCode.PERMISSION_DENIED
    finally: server.stop(0).wait()
    assert calls == ['canonical']


def test_security_and_capture_media_aliases_preserve_legacy_contracts():
    from contracts.public_compatibility import CompatibilityError, check_compatibility
    api = json.loads((ROOT/'contracts/api/openapi.json').read_text(encoding='utf-8'))
    layout = json.loads((ROOT/'contracts/capture/layout.json').read_text(encoding='utf-8'))
    baseline = json.loads((ROOT/'contracts/compatibility/2026-10-16/public-wire-baseline.json').read_text(encoding='utf-8'))
    schemes = api['components']['securitySchemes']
    schemes['ServerSession']['name'] = 'cirnet_session'
    schemes['CsrfToken']['name'] = 'X-CIRNET-CSRF'
    response = api['paths']['/api/captures/{capture_id}/payload']['get']['responses']['200']['content']
    legacy = 'application/vnd.netcircuit.capture; version=1'
    canonical = 'application/vnd.cirnet.capture; version=1'
    response[canonical] = deepcopy(response[legacy])
    check_compatibility(api, layout, baseline)
    response[canonical]['schema'] = {'type': 'integer'}
    with pytest.raises(CompatibilityError): check_compatibility(api, layout, baseline)
    response[canonical] = deepcopy(response[legacy])
    del response[legacy]
    with pytest.raises(CompatibilityError): check_compatibility(api, layout, baseline)
    api = json.loads((ROOT/'contracts/api/openapi.json').read_text(encoding='utf-8'))
    api['components']['securitySchemes']['ServerSession']['name'] = 'unknown_session'
    with pytest.raises(CompatibilityError): check_compatibility(api, layout, baseline)


def test_operation_generator_accepts_canonical_and_legacy_contract_metadata():
    from contracts.codegen.openapi import operation_bindings
    api = json.loads((ROOT/'contracts/api/openapi.json').read_text(encoding='utf-8'))
    expected = operation_bindings(api)
    for extension in ('x-cirnet-contract', 'x-netcircuit-contract'):
        document = deepcopy(api)
        for schema in document['components']['schemas'].values():
            if isinstance(schema, dict): schema.pop(extension, None)
        assert operation_bindings(document) == expected


def test_rpc_fallback_keeps_the_original_total_timeout(monkeypatch):
    import grpc
    import contracts.rpc_compat as compatibility
    times = iter([10.0, 12.5])
    monkeypatch.setattr(compatibility, 'monotonic', lambda: next(times))
    calls = []
    class Unsupported:
        def code(self): return grpc.StatusCode.UNIMPLEMENTED
        def add_done_callback(self, fn): fn(self)
    def continuation(details, request):
        calls.append((details, request))
        return Unsupported()
    details = compatibility._Details('/cirnet.hardware.v2.HardwareService/GetCapabilities', 5,
                                     (('key', 'value'),), None, True, None)
    compatibility._LegacyRouteFallback().intercept_unary_unary(continuation, details, 'request')
    assert len(calls) == 2
    legacy = calls[1][0]
    assert legacy.method == '/netcircuit.hardware.v2.HardwareService/GetCapabilities'
    assert legacy.timeout == 2.5
    assert legacy.metadata == details.metadata and legacy.wait_for_ready is True
    assert calls[1][1] == 'request'
