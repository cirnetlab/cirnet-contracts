"""Contracts release behavior without any consumer checkout or implementation."""
import hashlib
import importlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

import pytest

ROOT = Path(__file__).resolve().parents[1]


def test_distribution_has_no_consumer_source_and_separate_public_package():
    assert (ROOT / 'pyproject.toml').is_file(), 'Standalone Python distribution is missing'
    python = tomllib.loads((ROOT / 'pyproject.toml').read_text(encoding='utf-8'))
    assert python['project']['name'] == 'cirnet-contracts'
    assert python['project']['version'] == '0.1.1'
    for path in ('apps', 'services', 'simulator'):
        assert not (ROOT / path).exists(), path
    npm = json.loads((ROOT / 'packages/public-sdk/package.json').read_text())
    assert npm['name'] == '@tinihi-labs/contracts' and npm['version'] == '0.1.1'
    assert npm['files'] == ['dist']


def test_compatibility_export_requires_explicit_destination(monkeypatch):
    from contracts.codegen import generate_contracts
    monkeypatch.setattr(generate_contracts, '_publish_bytes', lambda *args: None)
    export_artifacts = generate_contracts.export_artifacts
    with pytest.raises(TypeError, match='output_root'):
        export_artifacts()


def test_prebuilt_export_preserves_frozen_browser_and_python_artifacts(tmp_path):
    assert (ROOT / 'contracts/resources.py').is_file(), 'Installed artifact resource API is missing'
    from contracts.resources import generated_artifacts
    from contracts.codegen.generate_contracts import export_artifacts
    expected = json.loads((ROOT / 'tests/fixtures/browser-projection-baseline.json').read_text())
    artifacts = generated_artifacts()
    assert set(expected['sha256']) <= set(artifacts)
    # Canonical names and additive compatibility validators change generated
    # bytes. Preserve the original projection evidence and its original digest
    # index independently; current outputs are checked by deterministic generation,
    # frozen wire compatibility and legacy/canonical reader tests.
    from zipfile import ZipFile
    with ZipFile(ROOT/'tests/fixtures/browser-projection-legacy.zip') as archive:
        assert set(archive.namelist()) == set(expected['sha256'])
        for path, digest in expected['sha256'].items():
            assert hashlib.sha256(archive.read(path)).hexdigest() == digest, path
    export_artifacts(output_root=tmp_path)
    export_artifacts(check=True, output_root=tmp_path)
    victim = tmp_path / 'apps/web/src/contracts/generated/dto.ts'
    victim.write_text('intentional-drift\n')
    with pytest.raises(SystemExit, match='Stale'):
        export_artifacts(check=True, output_root=tmp_path)
    assert victim.read_text() == 'intentional-drift\n'


def test_standalone_generation_is_deterministic_without_consumer_imports(tmp_path):
    assert (ROOT / 'contracts/codegen/generate_contracts.py').is_file(), 'Installed codegen is missing'
    from contracts.codegen.generate_contracts import generate
    first, second = tmp_path / 'one', tmp_path / 'two'
    generate(output_root=first)
    generate(output_root=second)
    def files(path):
        return {p.relative_to(path).as_posix(): p.read_bytes() for p in path.rglob('*') if p.is_file()}
    assert files(first) == files(second)
    assert 'contracts/browser/generated/dto.ts' in files(first)
    assert not any(path.startswith(('apps/', 'services/', 'simulator/')) for path in files(first))
    generate(check=True, output_root=first, node_tools=Path('packages/public-sdk'))
    assert not any(name == 'app' or name.startswith(('app.', 'hardware_service', 'cirnet_sim', 'netcircuit_sim')) for name in sys.modules)


def test_operation_bindings_come_from_frozen_document_and_reject_private_shape():
    assert (ROOT / 'contracts/codegen/openapi.py').is_file(), 'Pure operation generator is missing'
    from contracts.codegen.openapi import operation_bindings
    from contracts.resources import resource_json
    document = resource_json('api/openapi.json')
    actual = operation_bindings(document)
    assert 'apiOperations' in actual and 'ApiOperations' in actual
    broken = json.loads(json.dumps(document))
    operation = next(operation for methods in broken['paths'].values() for method, operation in methods.items()
                     if method == 'post' and 'requestBody' in operation)
    operation['requestBody']['content']['application/json']['schema'] = {'type': 'object'}
    with pytest.raises(ValueError, match='named canonical'):
        operation_bindings(broken)


def test_public_history_corpus_keeps_its_implicit_public_boundary():
    from contracts.resources import resource_json
    canonical = resource_json('fixtures/history-conformance.json')
    public = resource_json('browser/fixtures/history-conformance.json')
    assert canonical['cases']
    assert public == canonical


def test_packed_sdk_is_public_and_executes_without_generator_dependencies(tmp_path):
    package = ROOT / 'packages/public-sdk'
    assert (package / 'package.json').is_file(), 'Public SDK package is missing'
    result = subprocess.run(['node', str(ROOT / 'scripts/check_public_sdk.mjs'), '--output', str(tmp_path)],
                            cwd=ROOT, capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    assert 'PUBLIC_SDK_PACK_OK' in result.stdout


def test_sdk_gate_delegates_output_validation_before_creating_files(tmp_path):
    allocator = tmp_path/'rejecting-allocator.mjs'
    allocator.write_text("export function validateArtifactDirectory(){throw Error('ALLOCATOR_REJECTED_OUTPUT')}\n")
    output = tmp_path/'must-not-exist'
    result = subprocess.run(['node', str(ROOT/'scripts/check_public_sdk.mjs'), '--output', str(output)],
                            cwd=ROOT, capture_output=True, text=True,
                            env={**os.environ, 'CIRNET_ARTIFACT_PATHS_MODULE': allocator.as_uri()})
    assert result.returncode != 0 and 'ALLOCATOR_REJECTED_OUTPUT' in result.stderr
    assert not output.exists()


def test_browser_fixture_generation_ignores_checkout_newline_conversion(tmp_path, monkeypatch):
    import shutil
    from contracts.codegen import generate_contracts as generator
    copied = tmp_path / 'input' / 'contracts'
    shutil.copytree(ROOT / 'contracts', copied, ignore=shutil.ignore_patterns('__pycache__', '*.pyc'))
    monkeypatch.setattr(generator, 'CONTRACTS', copied)
    fixtures = [path for folder in ('design-models/fixtures', 'fixtures/capture')
                for path in (copied / folder).glob('*.json')]
    assert fixtures
    for path in fixtures:
        path.write_bytes(path.read_bytes().replace(b'\r\n', b'\n'))
    first, second = tmp_path / 'lf', tmp_path / 'crlf'
    generator.generate(output_root=first, node_tools=ROOT / 'packages/public-sdk')
    for path in fixtures:
        path.write_bytes(path.read_bytes().replace(b'\n', b'\r\n'))
    generator.generate(output_root=second, node_tools=ROOT / 'packages/public-sdk')
    def contents(root):
        return {path.relative_to(root).as_posix(): path.read_bytes()
                for path in root.rglob('*') if path.is_file()}
    assert contents(first) == contents(second)
