"""Keep generated contracts metadata and build files out of the source root.

Package metadata remains declared in pyproject.toml. Pip's setuptools backend
executes this hook for regular and editable installs, including fresh checkouts.
"""
from pathlib import Path
import os
import subprocess
import sys

from setuptools import setup
from setuptools.command.build_py import build_py


class BuildContracts(build_py):
    """Clean wheel/sdist builds generate local RPC bindings from canonical proto."""
    def run(self):
        super().run()
        subprocess.run([sys.executable, '-m', 'contracts.codegen.generate_hardware_rpc', '--output',
                        str(Path(self.build_lib) / 'contracts/generated')], check=True)


build_root = Path(os.environ.get("CIRNET_BUILD_ROOT", os.environ.get("NETCIRCUIT_BUILD_ROOT", ".build")))
metadata_dir = build_root / "metadata"
metadata_dir.mkdir(parents=True, exist_ok=True)

resource_files = [str(path.relative_to('contracts')).replace('\\', '/')
                  for path in Path('contracts').rglob('*')
                  if path.is_file() and path.suffix in {'.json', '.proto', '.mjs', '.js', '.ts'}]

setup(cmdclass={'build_py': BuildContracts}, package_data={'contracts': resource_files}, options={
    "egg_info": {"egg_base": str(metadata_dir)},
    "build": {"build_base": str(build_root)},
})
