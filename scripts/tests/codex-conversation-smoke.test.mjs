// SPDX-License-Identifier: Apache-2.0
import assert from 'node:assert/strict';
import { spawnSync, execFileSync } from 'node:child_process';
import { test } from 'node:test';
import { mkdtempSync, writeFileSync, rmSync, mkdirSync, symlinkSync, existsSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const script = new URL('../smoke-codex-conversation.mjs', import.meta.url);

test('live mode fails closed before any fixture starts without explicit operator opt-in', () => {
  const result = spawnSync(process.execPath, [script.pathname, '--live'], {
    encoding: 'utf8',
    env: { PATH: process.env.PATH },
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /live verification requires --allow-live/);
});

test('help documents the provider-free and gated live modes', () => {
  const result = spawnSync(process.execPath, [script.pathname, '--help'], {
    encoding: 'utf8',
    env: { PATH: process.env.PATH },
  });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /--fake/);
  assert.match(result.stdout, /--live --allow-live/);
});

test('live mode rejects a non-loopback broker origin before reading credentials or sending requests', () => {
  const directory = mkdtempSync(join(tmpdir(), 'Task9-live-guard-'));
  try {
    const packet = join(directory, 'packet.json');
    writeFileSync(packet, JSON.stringify({
      profileVersion: 'thought-khoral.agent-conversation.v1',
      activationApproved: true,
      providerCallsApproved: true,
      brokerOrigin: 'https://example.com',
    }), { mode: 0o600 });
    const result = spawnSync(process.execPath, [script.pathname, '--live', '--allow-live'], {
      encoding: 'utf8',
      env: { PATH: process.env.PATH, TASK9_LIVE_OPERATOR_PACKET: packet },
    });
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /brokerOrigin must be an exact loopback HTTP origin/);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

function privatePacket(directory) {
  const token = sub => 'synthetic.' + Buffer.from(JSON.stringify({ sub, n2n_role: 'human', padding: 'x'.repeat(100) })).toString('base64url') + '.synthetic';
  const mayaTokenFile = join(directory, 'maya.jwt');
  const leoTokenFile = join(directory, 'leo.jwt');
  writeFileSync(mayaTokenFile, token('11111111-1111-4111-8111-111111111111'), { mode: 0o600 });
  writeFileSync(leoTokenFile, token('22222222-2222-4222-8222-222222222222'), { mode: 0o600 });
  return {
    profileVersion: 'thought-khoral.agent-conversation.v1', activationApproved: true, providerCallsApproved: true,
    brokerOrigin: 'http://127.0.0.1:1', roomId: '33333333-3333-4333-8333-333333333333',
    agentId: '74686f75-6768-746b-686f-72616c000004', workerContainer: 'Task9-never-contact',
    factCode: 'Task9-fact-12345678', correctionCode: 'Task9-correction-12345678',
    interveningCode: 'Task9-intervening-12345678', targetedCanary: 'Task9-canary-12345678',
    contractTag: 'thought-khoral-agent-conversation-v1.0.0', cliVersion: '0.160.0',
    workerImageDigest: 'sha256:2d8bfade27802f910cf68e832722c93b4a2acc2addb825711e1223617a4cd385',
    revisions: {
      platform: execFileSync('git', ['-C', new URL('../..', import.meta.url).pathname, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
      broker: 'fd05cb48b8508e7939f9cdf9df275742a06fc4f8', mediator: '6c3d96b4763871b9addc9bc7223e71ee7d38abd9',
      worker: 'b0d43ec2b5b0c8da035d4ccff754545132b978d4', ui: 'e51d67e9e1a986601df6b5e1acf68aaf7ae0870d',
    }, mayaTokenFile, leoTokenFile, evidenceFile: join(directory, 'evidence.json'),
  };
}

for (const field of ['packet', 'mayaTokenFile', 'leoTokenFile', 'evidenceFile']) {
  for (const symlink of [false, true]) {
    test(`live rejects ${field} in another Git worktree${symlink ? ' through a symlinked parent' : ''} before external commands or writes`, () => {
      const directory = mkdtempSync(join(tmpdir(), 'Task9-private-guard-'));
      try {
        const repo = join(directory, 'other-repo');
        const worktree = join(directory, 'other-worktree');
        execFileSync('git', ['init', '-q', repo]);
        execFileSync('git', ['-C', repo, '-c', 'user.name=Task9', '-c', 'user.email=task9@example.invalid', 'commit', '--allow-empty', '-qm', 'synthetic']);
        execFileSync('git', ['-C', repo, 'worktree', 'add', '-q', '--detach', worktree]);
        const parent = symlink ? join(directory, 'alias') : worktree;
        if (symlink) symlinkSync(worktree, parent);
        const value = privatePacket(directory);
        let packet = join(directory, 'operator.json');
        if (field === 'packet') packet = join(parent, 'operator.json');
        else if (field === 'evidenceFile') value[field] = join(parent, 'missing-parent', 'evidence.json');
        else { value[field] = join(parent, 'human.jwt'); writeFileSync(value[field], 'Task9-invalid-token', { mode: 0o600 }); }
        writeFileSync(packet, JSON.stringify(value), { mode: 0o600 });
        const bin = join(directory, 'bin'); mkdirSync(bin);
        const contacted = join(directory, 'external-command');
        writeFileSync(join(bin, 'podman'), `#!/bin/sh\ntouch '${contacted}'\nexit 1\n`, { mode: 0o700 });
        const result = spawnSync(process.execPath, [script.pathname, '--live', '--allow-live'], {
          encoding: 'utf8', env: { PATH: `${bin}:${process.env.PATH}`, TASK9_LIVE_OPERATOR_PACKET: packet },
        });
        assert.notEqual(result.status, 0);
        assert.match(result.stderr, /outside all Git worktrees/);
        assert.equal(existsSync(contacted), false);
        assert.equal(existsSync(value.evidenceFile), false);
      } finally { rmSync(directory, { recursive: true, force: true }); }
    });
  }
}

test('live rejects an existing evidence file before commands and preserves its bytes', () => {
  const directory = mkdtempSync(join(tmpdir(), 'Task9-existing-evidence-'));
  try {
    const value = privatePacket(directory);
    writeFileSync(value.evidenceFile, 'Task9-original', { mode: 0o600 });
    const packet = join(directory, 'operator.json'); writeFileSync(packet, JSON.stringify(value), { mode: 0o600 });
    const result = spawnSync(process.execPath, [script.pathname, '--live', '--allow-live'], {
      encoding: 'utf8', env: { PATH: process.env.PATH, TASK9_LIVE_OPERATOR_PACKET: packet },
    });
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /evidenceFile must be a new file/);
    assert.equal(readFileSync(value.evidenceFile, 'utf8'), 'Task9-original');
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('alternate failures retain only known safe codes, fail the gate, and never assert account denial', async () => {
  const runner = await import(script);
  assert.equal(typeof runner.alternateFailure, 'function', 'safe alternate failure projection is required');
  for (const code of ['authentication_required', 'timeout', 'invalid_task_input', 'forbidden', 'conversation_interrupted', 'execution_failed', 'session_unavailable', 'runtime_unavailable']) {
    const check = runner.alternateFailure({ accepted: { taskId: 'Task9-task' }, view: { state: 'failed', failure: { code, message: 'PRIVATE-CONTENT' } } });
    assert.deepEqual(check, { label: 'alternate-settings', taskId: 'Task9-task', state: 'failed', failure: { code }, accountAccess: 'unclassified', gate: 'failed' });
    assert.equal(JSON.stringify(check).includes('PRIVATE-CONTENT'), false);
  }
  const unknown = runner.alternateFailure({ accepted: { taskId: 'Task9-task' }, view: { state: 'failed', failure: { code: 'PRIVATE-CODE' } } });
  assert.equal(unknown.failure.code, 'unknown');
  assert.equal(unknown.accountAccess, 'unclassified');
});
