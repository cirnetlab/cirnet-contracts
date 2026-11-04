"""Read-only installed-wheel gate; run with isolated Python outside source."""
import importlib.metadata
import importlib.resources
import json
from pathlib import Path
import sys


def main():
    assert sys.flags.isolated == 1, 'Use python -I for the installed-wheel probe.'
    import contracts
    distribution = importlib.metadata.distribution('cirnet-contracts')
    assert distribution.metadata['Name'] == 'cirnet-contracts'
    assert distribution.version == '0.1.1'
    package = Path(distribution.locate_file('contracts/__init__.py')).resolve().parent
    assert Path(contracts.__file__).resolve().parent == package
    assert Path(str(importlib.resources.files('contracts'))).resolve() == package
    assert not package.is_relative_to(Path(__file__).resolve().parents[1])
    from contracts.registry import check_manifest, validate_contract
    from contracts.boundary.models import AnySupportedCircuitGraph
    from contracts.capture_layout import CAPTURE_LAYOUT
    from contracts.boundary.capture_layout import CAPTURE_LAYOUT as GENERATED_LAYOUT
    from contracts.generated import rpc_v2_pb2, rpc_v2_pb2_grpc
    from contracts.codegen.openapi import operation_bindings
    from contracts.resources import generated_artifacts
    check_manifest()
    graph = {'schema_version': '1.0', 'circuit_id': 'installed-m1', 'modules': [],
             'connections': [], 'metadata': {'preserve': [None, False, 0, 10**50]}}
    assert AnySupportedCircuitGraph.model_validate(graph).model_dump(exclude_unset=True) == graph
    assert CAPTURE_LAYOUT == GENERATED_LAYOUT
    assert rpc_v2_pb2.ClockConfiguration(period_ns=4294967295).period_ns == 4294967295
    assert rpc_v2_pb2_grpc.HardwareServiceStub is not None
    validate_contract('MemoryImage', {'version': '1.0', 'word_bits': 8, 'depth': 1, 'data': [0]})
    resources = importlib.resources.files('contracts')
    snapshot = json.loads(resources.joinpath('api/openapi.json').read_text(encoding='utf-8'))
    assert operation_bindings(snapshot).encode() == generated_artifacts()['apps/web/src/contracts/generated/operations.ts']
    assert not any(name == 'app' or name.startswith(('app.', 'hardware_service', 'cirnet_sim', 'netcircuit_sim')) for name in sys.modules)
    print(json.dumps({'installed_version': distribution.version, 'python': sys.version.split()[0],
                      'package_root': str(package), 'isolated': True,
                      'offline_registry': True, 'rpc': True, 'operation_snapshot': True}, sort_keys=True))


if __name__ == '__main__':
    main()
