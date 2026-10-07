// SPDX-License-Identifier: Apache-2.0
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { execFileSync, spawnSync } from 'node:child_process';
import { cpSync, mkdtempSync, readFileSync, writeFileSync, rmSync, mkdirSync, existsSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import * as runner from '../smoke-codex-conversation.mjs';
const script = new URL('../smoke-codex-conversation.mjs', import.meta.url).pathname;
const candidateRoot = '/private/tmp/codex-conversation-defaults';
const vendor = join(candidateRoot, 'room-gateway/contracts/agent-conversation-v1.1-candidate');
const baseline = join(candidateRoot, 'room-gateway/contracts/agent-conversation-v1');
const pins = new URL('../fixtures/codex-conversation/defaults-candidate-pins.json', import.meta.url);
function temporary(fn) {
  const dir = mkdtempSync(join(tmpdir(), 'Task9-defaults-pins-'));
  try { return fn(dir); } finally { rmSync(dir, { recursive: true, force: true }); }
}
function override(key, value, fn) {
  const old = process.env[key]; process.env[key] = value;
  try { return fn(); } finally { if (old === undefined) delete process.env[key]; else process.env[key] = old; }
}
test('baseline sources remain exactly the retained three reviewed trees', () => {
  assert.equal(typeof runner.sourcePins, 'function', 'source preflight must be exported without runner side effects');
  assert.deepEqual(Object.keys(runner.sourcePins()).sort(), ['broker', 'mediator', 'worker']);
});
test('candidate sources include exact reviewed UI and contracts with matching artifact vendors', () => {
  assert.equal(typeof runner.sourcePins, 'function', 'candidate source preflight required');
  const paths = runner.sourcePins({ defaultsCandidate: true });
  assert.deepEqual(Object.keys(paths).sort(), ['broker', 'contracts', 'mediator', 'ui', 'worker']);
  assert.equal(paths.broker, join(candidateRoot, 'room-gateway'));
});
test('candidate flag is explicitly rejected by live before operator packet or credentials', () => {
  for (const args of [['--live', '--defaults-candidate'], ['--live', '--allow-live', '--defaults-candidate'], ['--fake', '--live', '--defaults-candidate']]) {
    const result = spawnSync(process.execPath, [script, ...args], { encoding: 'utf8', env: { PATH: process.env.PATH, TASK9_LIVE_OPERATOR_PACKET: '/does-not-exist' } });
    assert.notEqual(result.status, 0); assert.match(result.stderr, /defaults candidate.*live/i);
  }
});
for (const mutation of ['status', 'release URL', 'contract commit', 'UI head', 'broker head']) test(`candidate metadata rejects ${mutation} tamper`, () => temporary(dir => {
  assert.equal(typeof runner.verifyCandidatePins, 'function', 'anchored candidate metadata verifier required');
  const value = JSON.parse(readFileSync(pins));
  if (mutation === 'status') value.status = 'released';
  if (mutation === 'release URL') value.archiveUrl = 'https://example.invalid/release.tar';
  if (mutation === 'contract commit') value.sources.contracts.head = '0'.repeat(40);
  if (mutation === 'UI head') value.sources.ui.head = '0'.repeat(40);
  if (mutation === 'broker head') value.sources.broker.head = '0'.repeat(40);
  const target = join(dir, 'pins.json'); writeFileSync(target, JSON.stringify(value));
  assert.throws(() => runner.verifyCandidatePins(target), /candidate metadata/);
}));
for (const legacy of [false, true]) for (const mutation of ['changed', 'new', 'missing', 'lock']) test(`${legacy ? 'published' : 'candidate'} vendor rejects ${mutation} file`, () => temporary(dir => {
  assert.equal(typeof runner.verifyVendor, 'function', 'closed per-file vendor verification required');
  const target = join(dir, 'vendor'); cpSync(legacy ? baseline : vendor, target, { recursive: true });
  runner.verifyVendor(target, { legacy });
  const file = legacy ? 'schemas/turn.schema.json' : 'schemas/agent-conversation-v1/turn.schema.json';
  if (mutation === 'changed') writeFileSync(join(target, file), '{}');
  if (mutation === 'new') writeFileSync(join(target, 'new.json'), '{}');
  if (mutation === 'missing') rmSync(join(target, file));
  if (mutation === 'lock') writeFileSync(join(target, 'lock.json'), '{}');
  assert.throws(() => runner.verifyVendor(target, { legacy }), /vendor|lock|payload/);
}));
for (const source of ['BROKER', 'UI', 'CONTRACTS', 'MEDIATOR', 'WORKER']) test(`candidate rejects wrong ${source} revision`, () => temporary(dir => {
  execFileSync('git', ['init', '-q', dir]);
  execFileSync('git', ['-C', dir, '-c', 'user.name=Task9', '-c', 'user.email=task9@example.invalid', 'commit', '--allow-empty', '-qm', 'synthetic wrong source']);
  assert.equal(typeof runner.sourcePins, 'function');
  override(`TASK9_${source}_REPO`, dir, () => assert.throws(() => runner.sourcePins({ defaultsCandidate: true }), /revision mismatch/));
}));
test('candidate rejects dirty runtime source at the correct HEAD', () => temporary(dir => {
  assert.equal(typeof runner.sourcePins, 'function');
  execFileSync('git', ['clone', '-q', '--shared', join(candidateRoot, 'room-gateway'), dir]);
  writeFileSync(join(dir, 'untracked-source'), 'synthetic');
  override('TASK9_BROKER_REPO', dir, () => assert.throws(() => runner.sourcePins({ defaultsCandidate: true }), /dirty/));
}));
test('missing UI fails before podman or fixture allocation', () => temporary(dir => {
  const bin = join(dir, 'bin'); mkdirSync(bin); const touched = join(dir, 'touched');
  writeFileSync(join(bin, 'podman'), `#!/bin/sh\ntouch '${touched}'\nexit 1\n`, { mode: 0o700 });
  const result = spawnSync(process.execPath, [script, '--fake', '--defaults-candidate'], { encoding: 'utf8', env: { ...process.env, PATH: `${bin}:${process.env.PATH}`, TASK9_UI_REPO: join(dir, 'missing'), TMPDIR: dir } });
  assert.notEqual(result.status, 0); assert.match(result.stderr, /fatal: cannot change to '.*missing'/); assert.equal(existsSync(touched), false);
  assert.deepEqual(readdirSync(dir).filter(name => name.startsWith('Task9-codex-conversation-')), []);
}));

test('missing candidate archive or artifact fails the preflight verifier', () => temporary(dir => {
  const metadata = runner.verifyCandidatePins();
  const target = join(dir, metadata.contractCommit);
  cpSync(metadata.artifactPath, target, { recursive: true });
  const lock = JSON.parse(readFileSync(join(target, 'lock.json')));
  rmSync(join(target, lock.archiveFile));
  assert.throws(() => runner.verifyCandidateArtifact(join(candidateRoot, 'contracts'), target), /candidate rejected|No such file/);
  assert.throws(() => runner.verifyCandidateArtifact(join(candidateRoot, 'contracts'), join(dir, 'missing')), /candidate rejected|missing directory/);
}));

test('imports through stdin do not start the runner', () => {
  const result = spawnSync(process.execPath, ['--input-type=module', '-'], { encoding: 'utf8', input: `import { sourcePins } from ${JSON.stringify(new URL('../smoke-codex-conversation.mjs', import.meta.url).href)}; if (typeof sourcePins !== 'function') throw Error('missing');`, env: { PATH: process.env.PATH } });
  assert.equal(result.status, 0, result.stderr); assert.equal(result.stdout, '');
});
