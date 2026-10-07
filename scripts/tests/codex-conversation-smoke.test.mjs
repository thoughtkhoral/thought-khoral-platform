// SPDX-License-Identifier: Apache-2.0
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { test } from 'node:test';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
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
    }));
    const result = spawnSync(process.execPath, [script.pathname, '--live', '--allow-live'], {
      encoding: 'utf8',
      env: { PATH: process.env.PATH, TASK9_LIVE_OPERATOR_PACKET: packet },
    });
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /brokerOrigin must be an exact loopback HTTP origin/);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
