// SPDX-License-Identifier: Apache-2.0
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtempSync, mkdirSync, writeFileSync, existsSync, rmSync, readFileSync, copyFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

const moduleUrl = new URL('../lib/codex-packaged-profile.mjs', import.meta.url);
const api = () => import(moduleUrl);

test('fixed source profile loads only its reviewed bytes and rejects reanchored input', async () => {
  const { loadPackagedProfile } = await api();
  const loaded = loadPackagedProfile();
  assert.equal(loaded.profile.revisions.broker, '673294e66fed2b6e98d73e08cfe4a23a0e5e4d48');
  assert.equal(loaded.profile.revisions.ui, '29a940012adfeb796f59c5b89b00cab7a4d0df68');
  assert.equal(loaded.profile.worker.imageId, 'sha256:2dd78cdf5eddb9ca271611998ea44fe6d6cfde6cfca8ba7d1d6c8858240b540d');
  const directory = mkdtempSync(join(tmpdir(), 'packaged-anchor-'));
  try {
    mkdirSync(join(directory, 'lib'));
    mkdirSync(join(directory, 'fixtures/codex-conversation'), { recursive: true });
    copyFileSync(moduleUrl, join(directory, 'lib/profile.mjs'));
    const pin = join(directory, 'fixtures/codex-conversation/packaged-v1.1-pins.json');
    const value = JSON.parse(readFileSync(new URL('../fixtures/codex-conversation/packaged-v1.1-pins.json', import.meta.url)));
    value.revisions.broker = '8'.repeat(40); writeFileSync(pin, JSON.stringify(value));
    const isolated = await import(new URL('file://' + join(directory, 'lib/profile.mjs')));
    assert.throws(() => isolated.loadPackagedProfile(), /anchor/);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
const commit = digit => digit.repeat(40);
const sha = digit => digit.repeat(64);
function profile() {
  return {
    id: 'packaged-v1.1', profileVersion: 'thought-khoral.agent-conversation.v1', cliVersion: '0.160.0', platform: 'linux/arm64',
    revisions: { contracts: 'f1c8621810c8c121c1aafc8e5a342d6626c4b843', broker: commit('1'), mediator: commit('2'), worker: commit('3'), ui: commit('4'), memory: commit('5') },
    contract: { tag: 'thought-khoral-agent-conversation-v1.1.0', commit: 'f1c8621810c8c121c1aafc8e5a342d6626c4b843', archiveSha256: 'b0370e43627ce6044852df75bfbc4b749d904aa9cedcfc558c61bb4e07dc95ff', lockSha256: 'b7f3ad928c5c17be7e936da4959b7e7e8555bb4ed3602fa62f71a8721f042d8f' },
    worker: { imageId: 'sha256:' + sha('6'), cargoLockSha256: '1c70ff10992b000c93232155ca0032cab9b7a6c27a13ca4ac46eba862e864a34', cliSha256: '50b06603bdcdac39b714f5c3e68583c002b8ad8779ebfdaaf4932ff016b379c0', proofSha256: '91ab6c55e22236a578b494f071bbd82ce867a2e9aae4010526b7319165344169' },
  };
}
function packet(p) {
  return { acceptanceProfile: p.id, profileVersion: p.profileVersion, platform: p.platform,
    contractTag: p.contract.tag, cliVersion: p.cliVersion, workerImageDigest: p.worker.imageId,
    contract: structuredClone(p.contract), worker: structuredClone(p.worker), revisions: { ...p.revisions, platform: commit('7') } };
}

test('closed packaged metadata accepts reviewed-compatible synthetic input and rejects identity drift', async () => {
  const { validatePackagedProfile } = await api();
  assert.deepEqual(validatePackagedProfile(profile()), profile());
  const mutations = [
    p => { p.extra = true; }, p => { delete p.worker; }, p => { p.worker = []; },
    p => { p.platform = 'linux/amd64'; }, p => { p.cliVersion = '0.159.0'; },
    p => { p.id = 'arbitrary'; }, p => { p.revisions.broker = 'main'; }, p => { p.revisions.broker = '0'.repeat(40); },
    p => { p.revisions.extra = commit('8'); }, p => { delete p.revisions.memory; },
    p => { p.revisions.contracts = commit('8'); }, p => { p.contract.status = 'unreleased-local-candidate'; },
    p => { p.contract.tag = 'unreleased'; }, p => { p.contract.commit = commit('8'); },
    p => { p.contract.archiveSha256 = sha('8'); }, p => { p.contract.lockSha256 = sha('8'); },
    p => { p.worker.imageId = 'localhost/worker:dev'; }, p => { p.worker.cargoLockSha256 = sha('8'); },
    p => { p.worker.cliSha256 = sha('8'); }, p => { p.worker.proofSha256 = sha('8'); },
  ];
  for (const mutate of mutations) {
    const p = profile(); mutate(p); assert.throws(() => validatePackagedProfile(p));
  }
});

test('packet identities must match the selected authority and actual platform commit', async () => {
  const { validatePackagedPacket } = await api(); const p = profile();
  assert.doesNotThrow(() => validatePackagedPacket(p, packet(p), commit('7')));
  for (const mutate of [v => { v.contractTag = 'candidate'; }, v => { v.workerImageDigest = 'sha256:' + sha('8'); },
    v => { v.revisions.ui = commit('8'); }, v => { v.revisions.platform = commit('8'); },
    v => { v.contract.lockSha256 = sha('8'); }, v => { v.worker.extra = true; },
    v => { v.platform = 'linux/amd64'; }, v => { v.acceptanceProfile = 'historical'; }]) {
    const v = packet(p); mutate(v); assert.throws(() => validatePackagedPacket(p, v, commit('7')));
  }
});

test('source checks require exact clean platform and component commits before engine access', async () => {
  const { verifyPackagedSources } = await api(); const p = profile(); const v = packet(p);
  const keys = Object.keys(p.revisions);
  const env = Object.fromEntries(keys.map(key => [`TASK9_${key.toUpperCase()}_REPO`, '/synthetic/' + key]));
  const execute = (program, args) => {
    assert.equal(program, 'git'); assert.equal(args[0], '-C');
    const revision = args[1] === '/synthetic/platform' ? commit('7') : p.revisions[args[1].split('/').at(-1)];
    assert.ok(revision);
    if (args.slice(2).join(' ') === 'rev-parse HEAD') return revision;
    assert.equal(args.slice(2).join(' '), 'status --porcelain --untracked-files=all'); return '';
  };
  assert.deepEqual(verifyPackagedSources(p, v, '/synthetic/platform', execute, env), v.revisions);
  for (const bad of ['wrong', 'dirty']) {
    assert.throws(() => verifyPackagedSources(p, v, '/synthetic/platform', (program, args) => {
      if (args[1] === '/synthetic/broker') return args[2] === 'rev-parse' ? (bad === 'wrong' ? commit('8') : p.revisions.broker) : ' M dirty';
      return execute(program, args);
    }, env));
  }
});

test('actual worker preflight returns verified identity and refuses package, architecture, label and byte drift', async () => {
  const { verifyPackagedWorker } = await api(); const p = profile();
  const observations = [p.worker.imageId,
    JSON.stringify({ Id: p.worker.imageId, Os: 'linux', Architecture: 'arm64', Config: { Labels: { 'org.thoughtkhoral.source': p.revisions.worker, 'org.thoughtkhoral.cargo-lock': p.worker.cargoLockSha256 } } }),
    'tool policy verified; exposed tools: []\nworker package verified; codex-cli 0.160.0; uid/gid 10003; no inference',
    p.worker.cliSha256 + '  /usr/local/bin/codex\n' + p.worker.proofSha256 + '  /opt/thought-khoral-codex/tool-policy-proof-aarch64.json'];
  const commands = [
    ['inspect', '--format', '{{.Image}}', 'synthetic-worker'],
    ['image', 'inspect', '--format', '{{json .}}', p.worker.imageId],
    ['exec', 'synthetic-worker', '/usr/local/bin/thought-khoral-codex-agent', '--verify-package'],
    ['exec', 'synthetic-worker', 'sha256sum', '/usr/local/bin/codex', '/opt/thought-khoral-codex/tool-policy-proof-aarch64.json'],
  ];
  const run = (values) => {
    let index = 0;
    const result = verifyPackagedWorker(p, 'synthetic-worker', (program, args) => {
      assert.equal(program, 'podman'); assert.deepEqual(args, commands[index]); return values[index++];
    });
    assert.equal(index, 4); return result;
  };
  assert.deepEqual(run(observations), { imageId: p.worker.imageId, platform: p.platform, ...p.worker, sourceCommit: p.revisions.worker, packageVerified: true });
  assert.deepEqual(run([p.worker.imageId.slice(7), ...observations.slice(1)]), run(observations), 'Podman bare hex identity is the same exact reviewed image');
  for (const [at, value] of [[0, 'sha256:' + sha('8')], [1, observations[1].replace('arm64', 'amd64')],
    [1, observations[1].replace(p.revisions.worker, commit('8'))], [1, observations[1].replace(p.worker.cargoLockSha256, sha('8'))],
    [2, 'version only'], [3, observations[3].replace(p.worker.cliSha256, sha('8'))], [3, observations[3].replace(p.worker.proofSha256, sha('8'))]]) {
    const bad = [...observations]; bad[at] = value; assert.throws(() => run(bad));
  }
  assert.throws(() => verifyPackagedWorker(p, 'synthetic-worker', () => { throw new Error('PRIVATE-CONTENT'); }), error => !error.message.includes('PRIVATE-CONTENT'));
});

test('profile selection rejects invalid CLI combinations before private packet and engine access', async () => {
  const { selectAcceptanceProfile } = await api();
  assert.equal(selectAcceptanceProfile(['--live', '--allow-live']), null);
  assert.equal(selectAcceptanceProfile(['--live', '--allow-live', '--acceptance-profile', 'packaged-v1.1']), 'packaged-v1.1');
  const directory = mkdtempSync(join(tmpdir(), 'packaged-cli-'));
  try {
    const contacted = join(directory, 'engine-contact'); const evidence = join(directory, 'evidence.json');
    const bin = join(directory, 'bin'); mkdirSync(bin);
    for (const tool of ['git', 'podman']) writeFileSync(join(bin, tool), `#!${process.execPath}\nrequire('node:fs').writeFileSync(${JSON.stringify(contacted)},'contact');process.exit(1);\n`, { mode: 0o700 });
    for (const args of [
      ['--fake', '--acceptance-profile', 'packaged-v1.1'], ['--live', '--allow-live', '--acceptance-profile', 'unknown'],
      ['--live', '--allow-live', '--acceptance-profile'], ['--live', '--allow-live', '--acceptance-profile=packaged-v1.1'],
      ['--live', '--allow-live', '--acceptance-profile', 'packaged-v1.1', '--acceptance-profile', 'packaged-v1.1'],
      ['--live', '--allow-live', '--defaults-candidate', '--acceptance-profile', 'packaged-v1.1'],
    ]) {
      assert.throws(() => selectAcceptanceProfile(args));
      const result = spawnSync(process.execPath, [new URL('../smoke-codex-conversation.mjs', import.meta.url).pathname, ...args], {
        encoding: 'utf8', env: { PATH: bin, TASK9_LIVE_OPERATOR_PACKET: join(directory, 'never-read.json') },
      });
      assert.notEqual(result.status, 0); assert.match(result.stderr, /acceptance.profile|defaults candidate/);
      assert.equal(existsSync(contacted), false); assert.equal(existsSync(evidence), false);
    }
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('selected CLI validates exact sources and actual worker observations before the terminal gate', async () => {
  const { loadPackagedProfile } = await api(); const p = loadPackagedProfile().profile;
  const directory = mkdtempSync(join(tmpdir(), 'packaged-preflight-'));
  const token = sub => 'synthetic.' + Buffer.from(JSON.stringify({ sub, n2n_role: 'human', padding: 'x'.repeat(100) })).toString('base64url') + '.synthetic';
  try {
    const bin = join(directory, 'bin'); mkdirSync(bin);
    const log = join(directory, 'commands.jsonl');
    const environment = Object.fromEntries(Object.keys(p.revisions).map(key => [`TASK9_${key.toUpperCase()}_REPO`, join(directory, key)]));
    const revisions = Object.fromEntries(Object.entries(environment).map(([key, value]) => [value, p.revisions[key.slice(6, -5).toLowerCase()]]));
    const runnerRoot = new URL('../..', import.meta.url).pathname.replace(/\/$/, '');
    revisions[runnerRoot] = commit('7');
    const gitCode = `const fs=require('node:fs');const args=process.argv.slice(2);const revisions=${JSON.stringify(revisions)};if(args[0]!=='-C'||!revisions[args[1]])process.exit(2);if(args.slice(2).join(' ')==='rev-parse HEAD')console.log(revisions[args[1]]);else if(args.slice(2).join(' ')!=='status --porcelain --untracked-files=all')process.exit(3);`;
    writeFileSync(join(bin, 'git'), `#!${process.execPath}\n${gitCode}\n`, { mode: 0o700 });
    const outputs = [p.worker.imageId,
      JSON.stringify({ Id: p.worker.imageId, Os: 'linux', Architecture: 'arm64', Config: { Labels: { 'org.thoughtkhoral.source': p.revisions.worker, 'org.thoughtkhoral.cargo-lock': p.worker.cargoLockSha256 } } }),
      'tool policy verified; exposed tools: []\nworker package verified; codex-cli 0.160.0; uid/gid 10003; no inference',
      p.worker.cliSha256 + '  /usr/local/bin/codex\n' + p.worker.proofSha256 + '  /opt/thought-khoral-codex/tool-policy-proof-aarch64.json'];
    const podCode = `const fs=require('node:fs');const log=${JSON.stringify(log)};const n=fs.existsSync(log)?fs.readFileSync(log,'utf8').trim().split('\\n').length:0;fs.appendFileSync(log,JSON.stringify(process.argv.slice(2))+'\\n');console.log(${JSON.stringify(outputs)}[n]||'unexpected');`;
    writeFileSync(join(bin, 'podman'), `#!${process.execPath}\n${podCode}\n`, { mode: 0o700 });
    const evidenceFile = join(directory, 'evidence.json');
    const value = { ...packet(p), activationApproved: true, providerCallsApproved: true,
      brokerOrigin: 'http://127.0.0.1:1', roomId: '33333333-3333-4333-8333-333333333333',
      agentId: '74686f75-6768-746b-686f-72616c000004', workerContainer: 'synthetic-worker',
      factCode: 'Task9-fact-12345678', correctionCode: 'Task9-correction-12345678', interveningCode: 'Task9-intervening-12345678', targetedCanary: 'Task9-targeted-12345678',
      mayaTokenFile: join(directory, 'maya.token'), leoTokenFile: join(directory, 'leo.token'), evidenceFile };
    writeFileSync(value.mayaTokenFile, token('11111111-1111-4111-8111-111111111111'), { mode: 0o600 });
    writeFileSync(value.leoTokenFile, token('22222222-2222-4222-8222-222222222222'), { mode: 0o600 });
    const operator = join(directory, 'operator.json');
    const run = (historical = false) => spawnSync(process.execPath, [new URL('../smoke-codex-conversation.mjs', import.meta.url).pathname,
      '--live', '--allow-live', ...(historical ? [] : ['--acceptance-profile', 'packaged-v1.1'])], { encoding: 'utf8',
      env: { PATH: bin, ...environment, TASK9_LIVE_OPERATOR_PACKET: operator } });
    const wrong = structuredClone(value); wrong.revisions.ui = commit('8');
    writeFileSync(operator, JSON.stringify(wrong), { mode: 0o600 });
    let result = run(); assert.notEqual(result.status, 0); assert.match(result.stderr, /operator packet differs/);
    assert.equal(existsSync(log), false); assert.equal(existsSync(evidenceFile), false);
    writeFileSync(operator, JSON.stringify(value)); result = run();
    assert.notEqual(result.status, 0); assert.match(result.stderr, /interactive operator terminal/);
    const calls = readFileSync(log, 'utf8').trim().split('\n').map(line => JSON.parse(line));
    assert.equal(calls.length, 4); assert.equal(calls[2].at(-1), '--verify-package');
    assert.equal(calls[3][2], 'sha256sum'); assert.equal(existsSync(evidenceFile), false);
    const historical = { ...value,
      contractTag: 'thought-khoral-agent-conversation-v1.0.0',
      workerImageDigest: 'sha256:2d8bfade27802f910cf68e832722c93b4a2acc2addb825711e1223617a4cd385',
      revisions: { platform: commit('7'), broker: 'fd05cb48b8508e7939f9cdf9df275742a06fc4f8',
        mediator: '6c3d96b4763871b9addc9bc7223e71ee7d38abd9', worker: 'b0d43ec2b5b0c8da035d4ccff754545132b978d4', ui: 'e51d67e9e1a986601df6b5e1acf68aaf7ae0870d' } };
    for (const key of ['acceptanceProfile', 'platform', 'contract', 'worker']) delete historical[key];
    writeFileSync(operator, JSON.stringify(historical)); rmSync(log);
    writeFileSync(join(bin, 'podman'), `#!${process.execPath}\nrequire('node:fs').writeFileSync(${JSON.stringify(log)},'historical-inspect');console.log(${JSON.stringify(historical.workerImageDigest)});\n`, { mode: 0o700 });
    result = run(true); assert.notEqual(result.status, 0); assert.match(result.stderr, /interactive operator terminal/);
    assert.equal(readFileSync(log, 'utf8'), 'historical-inspect'); assert.equal(existsSync(evidenceFile), false);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
