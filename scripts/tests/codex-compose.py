#!/usr/bin/env python3
"""Exercise Compose interpolation and the opt-in boundary without activation."""
import os
import pathlib
import subprocess
import unittest
import tempfile
import shlex

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

class GeneratedArgv(unittest.TestCase):
    """Real installed Compose generator; engine replaced before any subprocess."""
    def generated(self, overlay=True):
        with tempfile.TemporaryDirectory(prefix='task9-compose-') as tmp:
            directory = pathlib.Path(tmp)
            engine = directory/'podman'
            engine.write_text('#!/usr/bin/env python3\nimport sys\nprint("[]" if "ps" in sys.argv else "")\n')
            engine.chmod(0o700)
            env = {k:v for k,v in os.environ.items() if not k.startswith('THOUGHT_KHORAL_')}
            env.update(THOUGHT_KHORAL_CODEX_IMAGE='sha256:'+'a'*64,
                THOUGHT_KHORAL_CODEX_PROVIDER_KEY_FILE=str(directory/'provider'),
                THOUGHT_KHORAL_CODEX_INVOCATION_KEY_FILE=str(directory/'invocation'),
                THOUGHT_KHORAL_CODEX_CATALOG_BRIDGE_KEY_FILE=str(directory/'bridge'),
                THOUGHT_KHORAL_CODEX_STATE_ROOT=str(directory/'state'),
                THOUGHT_KHORAL_CODEX_ADMISSION_EXPIRES_AT='2099-01-01T00:00:00Z',
                THOUGHT_KHORAL_CODEX_POLICY_JSON='{}',
                THOUGHT_KHORAL_CODEX_MODEL_POLICY_JSON='{}', THOUGHT_KHORAL_CODEX_MODELS='fixture')
            for name in ['provider', 'invocation', 'bridge']:
                (directory/name).write_text('synthetic-'+name)
            args = ['podman-compose', '--podman-path', str(engine), '--dry-run', '--verbose',
                    '-p', 'task9-compat-dryrun', '-f', str(ROOT/'compose.yaml')]
            if overlay: args += ['-f', str(ROOT/'compose.codex.yaml')]
            result = subprocess.run(args+['up', '--no-start', '--no-build'], cwd=directory,
                env=env, capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)
            creates = [shlex.split(line[line.index(str(engine)):]) for line in result.stderr.splitlines()
                       if str(engine)+' create ' in line]
            self.assertEqual(len(creates), 11 if overlay else 8, result.stderr)
            return {next(value.split('=', 1)[1] for value in args if value.startswith('--name=')):args for args in creates}

    def test_proxy_loopback_extra_host_is_one_complete_generated_argument(self):
        args = self.generated()['thought-khoral-codex-egress']
        hosts = [args[i+1] for i, value in enumerate(args) if value == '--add-host']
        self.assertEqual(hosts, ['thought-khoral-codex-provider-proxy:127.0.0.1'])

    def test_mediator_alias_is_on_namespace_owner_existing_internal_network(self):
        for overlay in [False, True]:
            with self.subTest(overlay=overlay):
                args = self.generated(overlay)['thought-khoral-agent-egress']
                networks = [value.split('=', 1)[1] for value in args if value.startswith('--network=')]
                internal = [value for value in networks if 'agent-internal' in value]
                self.assertEqual(len(internal), 1)
                self.assertIn('alias=thought-khoral-agent-gateway', internal[0])
                self.assertEqual(len(networks), 2 if overlay else 1)
                self.assertFalse(any(value.startswith('--publish') for value in args))

if __name__ == '__main__': unittest.main()
