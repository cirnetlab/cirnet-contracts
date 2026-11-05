"""A wheel must contain the same offline contract authority as the checkout."""
from pathlib import Path
import os
import subprocess
import sys
import tomllib
from zipfile import ZipFile

ROOT=Path(__file__).resolve().parents[2]




def test_wheel_contains_generated_rpc_and_offline_resources(tmp_path):
    version = tomllib.loads((ROOT / 'pyproject.toml').read_text(encoding='utf-8'))['project']['version']
    wheel_name = f'cirnet_contracts-{version}'
    result=subprocess.run([sys.executable,'-m','build','--wheel','--no-isolation','--outdir',str(tmp_path)],
                          cwd=ROOT,capture_output=True,text=True,
                          env={**os.environ,'TMP':str(tmp_path),'TEMP':str(tmp_path),'CIRNET_BUILD_ROOT':str(tmp_path/'build')})
    assert result.returncode==0,result.stdout+result.stderr
    wheel = tmp_path / f'{wheel_name}-py3-none-any.whl'
    assert list(tmp_path.glob('*.whl')) == [wheel]
    with ZipFile(wheel) as archive:
        files=set(archive.namelist())
        metadata=archive.read(f'{wheel_name}.dist-info/METADATA').decode('utf-8')
    assert 'Name: cirnet-contracts' in metadata.splitlines()
    assert f'Version: {version}' in metadata.splitlines()
    import json
    projections=json.loads((ROOT/'contracts/browser/artifacts.json').read_text(encoding='utf-8'))
    assert {'contracts/'+resource for resource in projections.values()} <= files
    assert {'contracts/manifest.json','contracts/profiles.json','contracts/fixtures/conformance.json','contracts/api/openapi.json',
            'contracts/design/visual.schema.json','contracts/public/common.schema.json',
            'contracts/compatibility/2026-10-16/baseline.json',
            'contracts/compatibility/2026-10-16/public-wire-baseline.json',
            'contracts/design-models/fixtures/physical-mounting-parity.json',
            'contracts/design-models/signal-domains.schema.json','contracts/design-models/signal-port-catalog.json',
            'contracts/capture/ncap.schema.json','contracts/capture/layout.json','contracts/fixtures/capture/conformance.json',
            'contracts/boundary/capture_layout.py','contracts/signal_domains.py','contracts/capture_validation.py',
            'contracts/hardware/v2/rpc_v2.proto','contracts/fpga-protocol/control.schema.json',
            'contracts/boundary/models.py','contracts/generated/rpc_v2_pb2.py','contracts/generated/rpc_v2_pb2_grpc.py'} <= files
