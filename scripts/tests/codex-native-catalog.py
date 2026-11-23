#!/usr/bin/env python3
"""Actual production worker adapter catalog; owned network-none, no model turn."""
import argparse
import datetime
import json
import io
import re
import sqlite3
import subprocess
import tempfile
import tarfile
import time
import uuid
from pathlib import Path


class FixtureFailure(Exception):
    """Only fixed diagnostic codes may escape this fixture."""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--image', required=True, help='reviewed immutable sha256 image ID')
    args = parser.parse_args()
    if not re.fullmatch(r'sha256:[0-9a-f]{64}', args.image):
        parser.error('an immutable sha256 image ID is required')
    prefix = 'task9-native-catalog-' + uuid.uuid4().hex[:16]
    volumes, containers = [], []
    observed_status = None
    evidence = None

    def pod(*argv, check=True, timeout=30):
        try:
            result = subprocess.run(['podman', *argv], capture_output=True, timeout=timeout)
        except (OSError, subprocess.TimeoutExpired):
            raise FixtureFailure('bounded_engine_operation_failed') from None
        if len(result.stdout) + len(result.stderr) > 131072:
            raise FixtureFailure('bounded_engine_output_exceeded')
        if check and result.returncode:
            raise FixtureFailure('engine_operation_failed')
        return result

    def request(path):
        # Only these two read-only authenticated HTTP endpoints are permitted.
        if path not in ['/.well-known/agent-card.json', '/control/v1/models']:
            raise FixtureFailure('unsupported_fixture_request')
        script = ('secret=$(cat /run/secrets/invocation); '
            'exec 3<>/dev/tcp/127.0.0.1/9091; '
            'printf "GET %s HTTP/1.1\\r\\nHost: localhost\\r\\nAuthorization: Bearer %s\\r\\nConnection: close\\r\\n\\r\\n" "$1" "$secret" >&3; '
            'cat <&3')
        return pod('exec', prefix + '-worker', '/bin/bash', '-eu', '-c', script,
                   'catalog-request', path, check=False, timeout=15)

    def parse_http(raw):
        if len(raw) > 65536:
            raise FixtureFailure('bounded_http_output_exceeded')
        try:
            header, body = raw.split(b'\r\n\r\n', 1)
            status = int(header.split(b'\r\n', 1)[0].split()[1])
            # Axum JSON bodies have Content-Length; never silently accept chunks.
            headers = dict(line.lower().split(b':', 1) for line in header.split(b'\r\n')[1:])
            if b'transfer-encoding' in headers or int(headers[b'content-length'].strip()) != len(body):
                raise ValueError()
            return status, json.loads(body)
        except (ValueError, KeyError, IndexError):
            raise FixtureFailure('invalid_http_response') from None

    try:
        identity = json.loads(pod('image', 'inspect', '--format', '{{json .}}', args.image).stdout)
        actual_id = identity['Id']
        if not actual_id.startswith('sha256:'):
            actual_id = 'sha256:' + actual_id
        if actual_id != args.image or identity['Os'] != 'linux':
            raise FixtureFailure('immutable_image_identity_mismatch')
        mounts = []
        for part, destination in [('native', '/var/lib/thought-khoral-codex/native'),
                                  ('receipts', '/var/lib/thought-khoral-codex/receipts'),
                                  ('secrets', '/run/secrets')]:
            name = prefix + '-' + part
            volumes.append(name)
            pod('volume', 'create', name)
            mounts += ['-v', name + ':' + destination + ':nocopy' + (',ro' if part == 'secrets' else '')]
        prepare_mounts = [value.replace(':nocopy,ro', ':nocopy') for value in mounts]
        prepare_name = prefix + '-prepare'
        containers.append(prepare_name)
        pod('run', '--rm', '--name', prepare_name, '--network', 'none', '--user', '0:0',
            '--cap-drop', 'ALL', '--cap-add', 'CHOWN', '--cap-add', 'FOWNER',
            '--read-only', *prepare_mounts, '--entrypoint', '/bin/bash', args.image, '-eu', '-c',
            'chown 10003:10003 /var/lib/thought-khoral-codex/native /var/lib/thought-khoral-codex/receipts; '
            'chmod 700 /var/lib/thought-khoral-codex/native /var/lib/thought-khoral-codex/receipts; '
            'printf synthetic-task9-provider-123456789 > /run/secrets/provider; '
            'printf synthetic-task9-invocation-987654321 > /run/secrets/invocation; '
            'chmod 444 /run/secrets/provider /run/secrets/invocation')
        expires = (datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(minutes=5)).isoformat()
        environment = {
            'THOUGHT_KHORAL_CODEX_ISOLATION_VERIFIED': '1',
            'THOUGHT_KHORAL_CODEX_PROVIDER_KEY_FILE': '/run/secrets/provider',
            'THOUGHT_KHORAL_CODEX_INVOCATION_KEY_FILE': '/run/secrets/invocation',
            'THOUGHT_KHORAL_CODEX_MODELS': 'gpt-6.1-sol',
            'THOUGHT_KHORAL_CODEX_ADMISSION_EXPIRES_AT': expires,
        }
        worker_name = prefix + '-worker'
        containers.append(worker_name)
        env_args = [arg for key, value in environment.items() for arg in ['-e', key + '=' + value]]
        pod('run', '-d', '--name', worker_name, '--network', 'none', '--user', '10003:10003',
            '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--read-only',
            '--tmpfs', '/tmp', *mounts, *env_args, args.image)
        actual = json.loads(pod('inspect', '--format', '{{json .}}', worker_name).stdout)
        if actual['HostConfig']['NetworkMode'] != 'none' or actual['Config']['User'] != '10003:10003' \
                or not actual['HostConfig']['ReadonlyRootfs'] or actual['HostConfig']['PortBindings'] \
                or actual['EffectiveCaps']:
            raise FixtureFailure('fixture_isolation_mismatch')
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            response = request('/.well-known/agent-card.json')
            if response.returncode == 0 and parse_http(response.stdout)[0] == 200:
                break
            time.sleep(0.1)
        else:
            raise FixtureFailure('worker_readiness_timeout')
        response = request('/control/v1/models')
        if response.returncode:
            raise FixtureFailure('adapter_catalog_request_failed')
        observed_status, catalog = parse_http(response.stdout)
        if observed_status != 200:
            raise FixtureFailure('adapter_catalog_http_refusal')
        if set(catalog) != {'profileVersion', 'catalogRevision', 'data', 'nextCursor'} \
                or catalog['profileVersion'] != 'thought-khoral.agent-conversation.v1' \
                or not isinstance(catalog['catalogRevision'], str) or not catalog['catalogRevision'] \
                or catalog['nextCursor'] is not None or len(catalog['data']) != 1:
            raise FixtureFailure('normalized_catalog_mismatch')
        model = catalog['data'][0]
        efforts = ['effort-' + value for value in ['low', 'medium', 'high', 'xhigh', 'max', 'ultra']]
        if set(model) != {'id', 'displayName', 'defaultReasoningEffort', 'supportedReasoningEfforts'} \
                or model['id'] != 'gpt-6.1-sol' or not isinstance(model['displayName'], str) \
                or model['defaultReasoningEffort'] != 'effort-low' \
                or [entry['id'] for entry in model['supportedReasoningEfforts']] != efforts \
                or any(set(entry) != {'id', 'description'} or not isinstance(entry['description'], str)
                       for entry in model['supportedReasoningEfforts']):
            raise FixtureFailure('normalized_model_options_mismatch')
        pod('stop', '--time', '5', worker_name, timeout=15)
        with tempfile.TemporaryDirectory(prefix=prefix + '-') as temporary:
            # Stream only owned synthetic SQLite files into caller-owned files.
            # Remote Podman cp to a directory may create an extra path component.
            archive = pod('cp', worker_name + ':/var/lib/thought-khoral-codex/receipts', '-').stdout
            with tarfile.open(fileobj=io.BytesIO(archive)) as contents:
                for member in contents.getmembers():
                    basename = Path(member.name).name
                    if member.isfile() and basename in {'worker.sqlite', 'worker.sqlite-wal', 'worker.sqlite-shm'}:
                        if member.size > 131072:
                            raise FixtureFailure('bounded_receipt_copy_exceeded')
                        (Path(temporary)/basename).write_bytes(contents.extractfile(member).read())
            with sqlite3.connect((Path(temporary)/'worker.sqlite').as_uri() + '?mode=ro', uri=True) as database:
                if database.execute('SELECT count(*) FROM receipts').fetchone()[0] != 0 \
                        or database.execute('SELECT count(*) FROM conversations').fetchone()[0] != 0:
                    raise FixtureFailure('unexpected_task_receipt_or_conversation')
        evidence = {'result': 'passed', 'imageId': args.image, 'platform': 'linux/' + identity['Architecture'],
            'observedStatus': observed_status, 'model': model['id'], 'reasoningEfforts': efforts,
            'network': 'none', 'taskReceipts': 0, 'conversations': 0, 'modelTurnsRequested': 0}
    except (FixtureFailure, KeyError, ValueError, sqlite3.Error, tarfile.TarError) as error:
        evidence = {'result': 'failed', 'imageId': args.image, 'observedStatus': observed_status,
            'reason': str(error) if isinstance(error, FixtureFailure) else 'fixture_validation_failed'}
    finally:
        cleanup_ok = True
        for name in reversed(containers):
            # --ignore makes already removed setup containers an expected condition.
            try:
                cleanup_ok &= pod('rm', '--force', '--ignore', name, check=False).returncode == 0
            except FixtureFailure:
                cleanup_ok = False
        for name in reversed(volumes):
            try:
                cleanup_ok &= pod('volume', 'rm', name, check=False).returncode == 0
            except FixtureFailure:
                cleanup_ok = False
        if not cleanup_ok:
            evidence = {'result': 'failed', 'imageId': args.image, 'observedStatus': observed_status,
                'reason': 'owned_fixture_cleanup_failed'}
    print(json.dumps(evidence, sort_keys=True))
    return 0 if evidence['result'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
