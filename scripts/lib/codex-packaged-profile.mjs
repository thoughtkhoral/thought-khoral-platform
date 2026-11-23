// SPDX-License-Identifier: Apache-2.0
import { isDeepStrictEqual } from 'node:util';
import { resolve } from 'node:path';
import { readFileSync, lstatSync } from 'node:fs';
import { createHash } from 'node:crypto';

const profileAnchors = new Map([
  ['packaged-v1.1', '7106f7e41a6bc13feab8a0b51fccde9b47fefca2d63980d551ed929eac1c1a71'],
  ['packaged-v1.1-runtime-compat', '14c5a784a1f6d8c944f25570f1b0dda741831178348ad7052ecb7301529d2f0c'],
]);

export function loadPackagedProfile(id = 'packaged-v1.1') {
  requireValue(profileAnchors.has(id), 'unknown acceptance profile');
  const profileUrl = new URL(`../fixtures/codex-conversation/${id}-pins.json`, import.meta.url);
  const sha256 = profileAnchors.get(id);
  requireValue(lstatSync(profileUrl).isFile(), 'acceptance profile anchor must be a regular file');
  const raw = readFileSync(profileUrl);
  requireValue(createHash('sha256').update(raw).digest('hex') === sha256, 'acceptance profile anchor mismatch');
  const profile = validatePackagedProfile(JSON.parse(raw));
  requireValue(profile.id === id, 'acceptance profile anchor identity mismatch');
  return { profile, sha256 };
}

const contract = {
  tag: 'thought-khoral-agent-conversation-v1.1.0',
  commit: 'f1c8621810c8c121c1aafc8e5a342d6626c4b843',
  archiveSha256: 'b0370e43627ce6044852df75bfbc4b749d904aa9cedcfc558c61bb4e07dc95ff',
  lockSha256: 'b7f3ad928c5c17be7e936da4959b7e7e8555bb4ed3602fa62f71a8721f042d8f',
};
const workerBytes = {
  cargoLockSha256: '1c70ff10992b000c93232155ca0032cab9b7a6c27a13ca4ac46eba862e864a34',
  cliSha256: '50b06603bdcdac39b714f5c3e68583c002b8ad8779ebfdaaf4932ff016b379c0',
  proofSha256: '91ab6c55e22236a578b494f071bbd82ce867a2e9aae4010526b7319165344169',
};
const components = {
  contracts: 'thought-khoral-contracts', broker: 'thought-khoral-room-gateway',
  mediator: 'thought-khoral-agent-gateway', worker: 'thought-khoral-codex-agent',
  ui: 'thought-khoral-workspace-ui', memory: 'thought-khoral-memory-engine',
};
const requireValue = (ok, message) => { if (!ok) throw new Error(message); };
const keysEqual = (value, keys) => value !== null && typeof value === 'object' && !Array.isArray(value)
  && isDeepStrictEqual(Object.keys(value).sort(), [...keys].sort());
const commit = value => typeof value === 'string' && /^[0-9a-f]{40}$/.test(value) && !/^0+$/.test(value);
const imageId = value => typeof value === 'string' && /^(sha256:)?[0-9a-f]{64}$/.test(value)
  ? (value.startsWith('sha256:') ? value : 'sha256:' + value) : null;

export function selectAcceptanceProfile(args) {
  if (!args.some(arg => arg.startsWith('--acceptance-profile'))) return null;
  requireValue(!args.includes('--defaults-candidate'), 'defaults candidate is forbidden with an acceptance profile');
  const index = args.indexOf('--acceptance-profile');
  requireValue(args.length === 4 && index >= 0 && profileAnchors.has(args[index + 1])
    && args.filter(arg => arg === '--acceptance-profile').length === 1
    && args.includes('--live') && args.includes('--allow-live'),
  'acceptance profile requires exactly --live --allow-live --acceptance-profile <reviewed-profile>');
  return args[index + 1];
}

export function validatePackagedProfile(p) {
  requireValue(keysEqual(p, ['id', 'profileVersion', 'cliVersion', 'platform', 'revisions', 'contract', 'worker']), 'closed acceptance profile required');
  requireValue(profileAnchors.has(p.id) && p.profileVersion === 'thought-khoral.agent-conversation.v1'
    && p.cliVersion === '0.160.0' && p.platform === 'linux/arm64', 'acceptance profile identity mismatch');
  requireValue(keysEqual(p.revisions, Object.keys(components)) && Object.values(p.revisions).every(commit), 'exact component revisions required');
  requireValue(isDeepStrictEqual(p.contract, contract) && p.revisions.contracts === contract.commit, 'published release provenance mismatch');
  requireValue(keysEqual(p.worker, ['imageId', ...Object.keys(workerBytes)])
    && typeof p.worker.imageId === 'string' && /^sha256:[0-9a-f]{64}$/.test(p.worker.imageId)
    && p.worker.imageId !== 'sha256:' + '0'.repeat(64)
    && Object.entries(workerBytes).every(([key, value]) => p.worker[key] === value), 'worker provenance mismatch');
  return p;
}

export function validatePackagedPacket(p, value, platformRevision) {
  validatePackagedProfile(p);
  if (p.id === 'packaged-v1.1-runtime-compat') {
    requireValue(typeof value.postgresContainer === 'string'
      && /^[A-Za-z0-9][A-Za-z0-9_.-]{0,127}$/.test(value.postgresContainer), 'invalid database container in compatibility packet');
  }
  requireValue(commit(platformRevision) && value.acceptanceProfile === p.id && value.profileVersion === p.profileVersion
    && value.platform === p.platform && value.contractTag === p.contract.tag && value.cliVersion === p.cliVersion
    && value.workerImageDigest === p.worker.imageId && isDeepStrictEqual(value.contract, p.contract)
    && isDeepStrictEqual(value.worker, p.worker) && isDeepStrictEqual(value.revisions, { ...p.revisions, platform: platformRevision }),
  'operator packet differs from reviewed acceptance profile');
}

export function verifyPackagedSources(p, packet, root, execute, environment = process.env) {
  const actual = execute('git', ['-C', root, 'rev-parse', 'HEAD']);
  validatePackagedPacket(p, packet, actual);
  requireValue(execute('git', ['-C', root, 'status', '--porcelain', '--untracked-files=all']) === '', 'platform source worktree is dirty');
  for (const [key, repository] of Object.entries(components)) {
    const directory = resolve(environment[`TASK9_${key.toUpperCase()}_REPO`] || resolve(root, '..', repository));
    requireValue(execute('git', ['-C', directory, 'rev-parse', 'HEAD']) === p.revisions[key], `${key} source revision mismatch`);
    requireValue(execute('git', ['-C', directory, 'status', '--porcelain', '--untracked-files=all']) === '', `${key} source worktree is dirty`);
  }
  return { ...p.revisions, platform: actual };
}

export function verifyPackagedWorker(p, name, execute) {
  validatePackagedProfile(p);
  requireValue(typeof name === 'string' && /^[A-Za-z0-9_.-]{1,128}$/.test(name), 'invalid packaged worker container');
  try {
    requireValue(imageId(execute('podman', ['inspect', '--format', '{{.Image}}', name])) === p.worker.imageId, 'image mismatch');
    const image = JSON.parse(execute('podman', ['image', 'inspect', '--format', '{{json .}}', p.worker.imageId]));
    const id = imageId(image.Id);
    requireValue(id === p.worker.imageId && image.Os === 'linux' && image.Architecture === 'arm64'
      && image.Config?.Labels?.['org.thoughtkhoral.source'] === p.revisions.worker
      && image.Config?.Labels?.['org.thoughtkhoral.cargo-lock'] === p.worker.cargoLockSha256, 'image provenance mismatch');
    const markers = execute('podman', ['exec', name, '/usr/local/bin/thought-khoral-codex-agent', '--verify-package']).split('\n');
    requireValue(markers.includes('tool policy verified; exposed tools: []')
      && markers.includes('worker package verified; codex-cli 0.160.0; uid/gid 10003; no inference'), 'package admission failed');
    const expected = [
      [p.worker.cliSha256, '/usr/local/bin/codex'],
      [p.worker.proofSha256, '/opt/thought-khoral-codex/tool-policy-proof-aarch64.json'],
    ];
    const lines = execute('podman', ['exec', name, 'sha256sum', ...expected.map(([, path]) => path)]).split('\n');
    requireValue(lines.length === 2 && lines.every((line, index) => {
      const match = /^([a-f0-9]{64})\s+(\S+)$/.exec(line);
      return match && match[1] === expected[index][0] && match[2] === expected[index][1];
    }), 'installed package bytes mismatch');
    return { imageId: p.worker.imageId, platform: p.platform, ...p.worker, sourceCommit: p.revisions.worker, packageVerified: true };
  } catch {
    throw new Error('packaged worker identity/package verification failed');
  }
}
