"""Exercise Python callers against the same packaged Node policy."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import cirnet_artifact_paths as allocator

SOURCE = Path(os.environ['ALLOCATOR_TEST_SOURCE']).resolve()
SCRATCH = Path(os.environ['ALLOCATOR_TEST_SCRATCH']).resolve()
CONTEXT = {'owner': 'python-bridge-tests', 'roots': [str(SOURCE)]}


class PythonBridgeTests(unittest.TestCase):
    def setUp(self):
        self.parent = Path(tempfile.mkdtemp(prefix='python-', dir=SCRATCH))
        self.environment = patch.dict(os.environ)
        self.environment.start()
        for name in ('CIRNET_ARTIFACTS_DIR', 'CIRNET_ARTIFACTS_SESSION', 'CIRNET_ARTIFACTS_CONTEXT',
                     'NETCIRCUIT_ARTIFACTS_DIR', 'NETCIRCUIT_ARTIFACTS_SESSION', 'NETCIRCUIT_ARTIFACTS_CONTEXT'):
            os.environ.pop(name, None)

    def tearDown(self):
        self.environment.stop()

    def test_legacy_import_is_a_leaf_and_exports_the_same_functions(self):
        import netcircuit_artifact_paths as legacy
        self.assertIs(legacy.artifact_directory, allocator.artifact_directory)
        self.assertIs(legacy.cleanup_artifact_session, allocator.cleanup_artifact_session)

    def test_explicit_allocation_does_not_reassign_parent_session(self):
        session = allocator.artifact_directory('python', directory=self.parent, context=CONTEXT)
        self.assertEqual(session.parent.parent, self.parent)
        self.assertNotIn('CIRNET_ARTIFACTS_SESSION', os.environ)
        self.assertNotIn('CIRNET_ARTIFACTS_CONTEXT', os.environ)
        allocator.cleanup_artifact_session(session.parent, context=CONTEXT)
        self.assertFalse(session.exists())

    def test_python_node_children_inherit_the_same_context_and_session(self):
        os.environ['CIRNET_ARTIFACTS_DIR'] = str(self.parent)
        session = allocator.artifact_directory(context=CONTEXT)
        try:
            evidence = allocator.artifact_directory('python-evidence')
            self.assertEqual(evidence.parent, session)
            self.assertEqual(os.environ['CIRNET_ARTIFACTS_SESSION'], os.environ['NETCIRCUIT_ARTIFACTS_SESSION'])
            self.assertEqual(os.environ['CIRNET_ARTIFACTS_CONTEXT'], os.environ['NETCIRCUIT_ARTIFACTS_CONTEXT'])
            child = subprocess.run([sys.executable, '-B', '-c',
                'from cirnet_artifact_paths import artifact_directory; print(artifact_directory("child"))'],
                capture_output=True, text=True, check=True, cwd=self.parent,
                env={**os.environ, 'PYTHONPATH': str(Path(allocator.__file__).parent.parent)})
            self.assertEqual(Path(child.stdout.strip()), session / 'child')
            with self.assertRaises(RuntimeError):
                allocator.cleanup_artifact_session(session, context={'owner': 'other', 'roots': [str(SOURCE)]})
            self.assertTrue(session.exists())
            with self.assertRaises(RuntimeError):
                allocator.validate_artifact_directory(SOURCE / 'docs')
        finally:
            allocator.cleanup_artifact_session(session)
        self.assertNotIn('CIRNET_ARTIFACTS_SESSION', os.environ)
        self.assertFalse(session.exists())


if __name__ == '__main__':
    unittest.main()
