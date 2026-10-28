#!/usr/bin/env python3
"""Check that CI clones/checks out the compatibility manifest's exact sources."""
import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


class CiCompatibilityTests(unittest.TestCase):
    def test_native_setup_uses_manifest_sources(self):
        compat = json.loads((ROOT / 'compatibility.json').read_text())
        workflow = (ROOT / '.github/workflows/ci.yml').read_text()
        commands = []
        for line in workflow.splitlines():
            if not line.startswith('          '):
                continue
            command = line.strip()
            if command.startswith(('git ', 'VEINMINER_')) and 'VEINMINER_STAGE' not in command:
                commands.append(command)
        script = 'git() { printf "%s\\n" "$*"; };\n' + '\n'.join(commands)
        actual = subprocess.check_output(['bash', '-eu', '-c', script], cwd=ROOT, text=True)
        self.assertIn('clone ' + compat['horizon_mixins_repository'] + ' ../horizon-mixins\n', actual)
        self.assertIn('-C ../horizon-mixins checkout ' + compat['horizon_mixins_revision'] + '\n', actual)
        self.assertIn('clone --filter=blob:none ' + compat['horizon_repository'] + ' ../horizon\n', actual)
        self.assertIn('-C ../horizon checkout ' + compat['horizon_revision'] + '\n', actual)


if __name__ == '__main__':
    unittest.main()
