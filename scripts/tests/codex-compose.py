#!/usr/bin/env python3
"""Exercise Compose interpolation and the opt-in boundary without activation."""
import os
import pathlib
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]

class Composition(unittest.TestCase):
    def config(self, overlay=False, **env):
        args = ['podman-compose', '-f', 'compose.yaml']
        if overlay:
            args += ['-f', 'compose.codex.yaml']
        return subprocess.run(args + ['config', '--services'], cwd=ROOT,
                              env={k:v for k,v in os.environ.items() if not k.startswith('THOUGHT_KHORAL_CODEX_')} | env,
                              capture_output=True, text=True)

    def test_default_needs_no_codex_credentials_and_has_no_codex_services(self):
        result = self.config()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('codex', result.stdout)
        self.assertEqual(len(result.stdout.splitlines()), 8)

    def test_overlay_without_credentials_is_rejected(self):
        result = self.config(True)
        self.assertNotEqual(result.returncode, 0)

    def test_explicit_opt_in_adds_isolated_services(self):
        result = self.config(True,
            THOUGHT_KHORAL_CODEX_IMAGE='localhost/fixture@sha256:'+'a'*64,
            THOUGHT_KHORAL_CODEX_PROVIDER_KEY_FILE='/tmp/task8-provider',
            THOUGHT_KHORAL_CODEX_INVOCATION_KEY_FILE='/tmp/task8-invocation',
            THOUGHT_KHORAL_CODEX_CATALOG_BRIDGE_KEY_FILE='/tmp/task8-bridge',
            THOUGHT_KHORAL_CODEX_STATE_ROOT='/tmp/task8-state',
            THOUGHT_KHORAL_CODEX_ADMISSION_EXPIRES_AT='2099-01-01T00:00:00Z',
            THOUGHT_KHORAL_CODEX_POLICY_JSON='{}',
            THOUGHT_KHORAL_CODEX_MODEL_POLICY_JSON='{}',
            THOUGHT_KHORAL_CODEX_MODELS='fixture')
        self.assertEqual(result.returncode, 0, result.stderr)
        for name in ['thought-khoral-codex-agent', 'thought-khoral-codex-egress', 'thought-khoral-codex-provider-proxy']:
            self.assertIn(name, result.stdout.splitlines())

if __name__ == '__main__': unittest.main()
