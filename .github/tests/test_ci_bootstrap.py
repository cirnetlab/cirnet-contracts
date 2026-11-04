from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def test_standalone_ci_publishes_source_allocator_for_node_checks():
    workflow = (ROOT / '.github/workflows/contracts.yml').read_text()
    allocation = workflow.split('- name: Allocate verification output', 1)[1].split('- name:', 1)[0]
    match = re.search(r'echo "CIRNET_ARTIFACT_PATHS_MODULE=\$\(node --input-type=module -e \'([^\']+)\'\)" >> "\$GITHUB_ENV"', allocation)
    assert match, 'CI must export its verified source allocator URI before pytest/SDK checks'
    uri = subprocess.check_output(['node', '--input-type=module', '-e', match.group(1)], cwd=ROOT, text=True).strip()
    expected = (ROOT / 'tools/artifact-paths/cirnet_artifact_paths/artifact-paths.mjs').as_uri()
    assert uri == expected
    result = subprocess.run(['node', '--input-type=module', '-e',
                             "const m=await import(process.argv[1]);if(typeof m.validateArtifactDirectory!=='function')process.exit(2)", uri],
                            cwd=ROOT, text=True, capture_output=True)
    assert result.returncode == 0, result.stdout + result.stderr
