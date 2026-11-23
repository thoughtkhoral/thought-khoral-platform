// SPDX-License-Identifier: Apache-2.0
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createHash } from 'node:crypto';
import { readFileSync, mkdtempSync, mkdirSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { loadPackagedProfile, selectAcceptanceProfile, validatePackagedPacket } from '../lib/codex-packaged-profile.mjs';
const id = 'packaged-v1.1-runtime-compat';
const source = '423d4e84985eb535caa66c7522c4191c6dbccad6';
const image = 'sha256:1ac229b25167845b4d7cec217304f3064b02d65d60c53f35b07c5ae3e3683b3b';
const platform = '7'.repeat(40);
const packet = p => ({ acceptanceProfile: p.id, profileVersion: p.profileVersion, platform: p.platform,
  contractTag: p.contract.tag, cliVersion: p.cliVersion, workerImageDigest: p.worker.imageId,
  contract: structuredClone(p.contract), worker: structuredClone(p.worker), revisions: { ...p.revisions, platform },
  ...(p.id === id ? { postgresContainer: 'task9-postgres-clone' } : {}) });

test('separate anchored compatibility profile changes only approved worker source/image and identity', () => {
  const old = loadPackagedProfile();
  const current = loadPackagedProfile(id);
  assert.equal(current.profile.id, id);
  assert.equal(current.profile.revisions.worker, source);
  assert.equal(current.profile.worker.imageId, image);
  assert.deepEqual(current.profile, { ...old.profile, id, revisions: { ...old.profile.revisions, worker: source },
    worker: { ...old.profile.worker, imageId: image } });
  assert.equal(old.sha256, '7106f7e41a6bc13feab8a0b51fccde9b47fefca2d63980d551ed929eac1c1a71');
  const raw = readFileSync(new URL('../fixtures/codex-conversation/packaged-v1.1-runtime-compat-pins.json', import.meta.url));
  assert.equal(current.sha256, createHash('sha256').update(raw).digest('hex'));
  assert.throws(() => loadPackagedProfile('arbitrary'));
});

test('compatibility selection remains explicit and rejects malformed or mixed opt-in', () => {
  assert.equal(selectAcceptanceProfile(['--live', '--allow-live', '--acceptance-profile', id]), id);
  for (const args of [ ['--fake', '--acceptance-profile', id], ['--live', '--acceptance-profile', id],
    ['--live', '--allow-live', '--acceptance-profile=' + id],
    ['--live', '--allow-live', '--defaults-candidate', '--acceptance-profile', id],
    ['--live', '--allow-live', '--acceptance-profile', id, '--acceptance-profile', 'packaged-v1.1'] ])
    assert.throws(() => selectAcceptanceProfile(args));
});

test('cross-profile packets and missing or malformed compatibility database targets are rejected', () => {
  const old = loadPackagedProfile().profile;
  const current = loadPackagedProfile(id).profile;
  assert.doesNotThrow(() => validatePackagedPacket(current, packet(current), platform));
  assert.throws(() => validatePackagedPacket(current, packet(old), platform));
  assert.throws(() => validatePackagedPacket(old, packet(current), platform));
  for (const target of [undefined, null, '', '-bad', 'x;bad', 'x/y', 'x'.repeat(129)]) {
    assert.throws(() => validatePackagedPacket(current, { ...packet(current), postgresContainer: target }, platform));
  }
});

test('only compatibility profile invokes the bounded schema hook and refuses missing schema', async () => {
  const { verifyPackagedSchema } = await import('../lib/codex-runtime-compat.mjs');
  let calls = 0;
  const current = loadPackagedProfile(id).profile;
  const old = loadPackagedProfile().profile;
  const schema = readFileSync(new URL('../fixtures/codex-conversation/schema-0006-0007.json', import.meta.url), 'utf8');
  assert.equal(verifyPackagedSchema(old, packet(old), () => { calls++; }), null);
  assert.equal(calls, 0);
  assert.deepEqual(verifyPackagedSchema(current, packet(current), (program, argv, options) => {
    calls++;
    assert.equal(program, 'podman'); assert.deepEqual(argv.slice(0, 2), ['exec', 'task9-postgres-clone']);
    assert.equal(options.timeout, 15000); assert.equal(options.maxBuffer, 131072);
    return schema;
  }), { conversationSchemaReady: true, migrations: ['0006', '0007'] });
  assert.equal(calls, 1);
  assert.throws(() => verifyPackagedSchema(current, packet(current), () => '{}'), /schema readiness failed/);
});

test('selected compatibility CLI checks schema before terminal/provider gate and rejects profile mismatch early', () => {
  const p = loadPackagedProfile(id).profile;
  const directory = mkdtempSync(join(tmpdir(), 'compat-preflight-'));
  const token = sub => 'synthetic.' + Buffer.from(JSON.stringify({ sub, n2n_role: 'human', padding: 'x'.repeat(100) })).toString('base64url') + '.synthetic';
  try {
    const bin = join(directory, 'bin'); mkdirSync(bin);
    const log = join(directory, 'engine.jsonl');
    const env = Object.fromEntries(Object.keys(p.revisions).map(key => [`TASK9_${key.toUpperCase()}_REPO`, join(directory, key)]));
    const revisions = Object.fromEntries(Object.entries(env).map(([key, value]) => [value, p.revisions[key.slice(6, -5).toLowerCase()]]));
    const runnerRoot = new URL('../..', import.meta.url).pathname.replace(/\/$/, ''); revisions[runnerRoot] = platform;
    writeFileSync(join(bin, 'git'), `#!${process.execPath}\nconst a=process.argv.slice(2),r=${JSON.stringify(revisions)};if(a[0]!=='-C'||!r[a[1]])process.exit(2);if(a.slice(2).join(' ')==='rev-parse HEAD')console.log(r[a[1]]);else if(a.slice(2).join(' ')!=='status --porcelain --untracked-files=all')process.exit(3);\n`, { mode: 0o700 });
    const schema = readFileSync(new URL('../fixtures/codex-conversation/schema-0006-0007.json', import.meta.url), 'utf8');
    const outputs = [p.worker.imageId, JSON.stringify({ Id: image, Os: 'linux', Architecture: 'arm64',
      Config: { Labels: { 'org.thoughtkhoral.source': source, 'org.thoughtkhoral.cargo-lock': p.worker.cargoLockSha256 } } }),
      'tool policy verified; exposed tools: []\nworker package verified; codex-cli 0.160.0; uid/gid 10003; no inference',
      p.worker.cliSha256 + '  /usr/local/bin/codex\n' + p.worker.proofSha256 + '  /opt/thought-khoral-codex/tool-policy-proof-aarch64.json', schema];
    writeFileSync(join(bin, 'podman'), `#!${process.execPath}\nconst fs=require('node:fs'),log=${JSON.stringify(log)},a=process.argv.slice(2),n=fs.existsSync(log)?fs.readFileSync(log,'utf8').trim().split('\\n').length:0;fs.appendFileSync(log,JSON.stringify(a)+'\\n');console.log((process.env.MISSING_SCHEMA&&n===4)?'{}':${JSON.stringify(outputs)}[n]||'unexpected');\n`, { mode: 0o700 });
    const value = { ...packet(p), activationApproved: true, providerCallsApproved: true,
      brokerOrigin: 'http://127.0.0.1:1', roomId: '33333333-3333-4333-8333-333333333333',
      agentId: '74686f75-6768-746b-686f-72616c000004', workerContainer: 'synthetic-worker',
      factCode: 'Task9-fact-12345678', correctionCode: 'Task9-correction-12345678',
      interveningCode: 'Task9-intervening-12345678', targetedCanary: 'Task9-targeted-12345678',
      mayaTokenFile: join(directory, 'maya.token'), leoTokenFile: join(directory, 'leo.token'), evidenceFile: join(directory, 'evidence.json') };
    writeFileSync(value.mayaTokenFile, token('11111111-1111-4111-8111-111111111111'), { mode: 0o600 });
    writeFileSync(value.leoTokenFile, token('22222222-2222-4222-8222-222222222222'), { mode: 0o600 });
    const operator = join(directory, 'operator.json');
    const run = (extra = {}) => spawnSync(process.execPath, [new URL('../smoke-codex-conversation.mjs', import.meta.url).pathname,
      '--live', '--allow-live', '--acceptance-profile', id], { encoding: 'utf8',
      env: { PATH: bin, ...env, ...extra, TASK9_LIVE_OPERATOR_PACKET: operator } });
    writeFileSync(operator, JSON.stringify(value), { mode: 0o600 });
    let result = run();
    assert.match(result.stderr, /interactive operator terminal/);
    let observed = readFileSync(log, 'utf8').trim().split('\n').map(JSON.parse);
    assert.equal(observed.length, 5); assert.deepEqual(observed[4].slice(0, 2), ['exec', 'task9-postgres-clone']);
    assert.equal(existsSync(value.evidenceFile), false);
    rmSync(log);
    result = run({ MISSING_SCHEMA: '1' }); assert.match(result.stderr, /schema readiness failed/);
    assert.equal(existsSync(value.evidenceFile), false); rmSync(log);
    for (const invalid of [{ ...value, acceptanceProfile: 'packaged-v1.1' }, { ...value, postgresContainer: 'bad;name' }]) {
      writeFileSync(operator, JSON.stringify(invalid)); result = run();
      assert.notEqual(result.status, 0); assert.equal(existsSync(log), false);
      assert.equal(existsSync(value.evidenceFile), false);
    }
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
