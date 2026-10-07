// SPDX-License-Identifier: Apache-2.0
// Actual composed failures and outer signals: own Task9 resources only; no provider.
import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { test } from 'node:test';
import { mkdtempSync, readdirSync, existsSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
const script = new URL('../smoke-codex-conversation.mjs', import.meta.url);
function ownedProcesses(directory) {
  return execFileSync('ps', ['-axo', 'pid=,ppid=,pgid=,command='], { encoding: 'utf8' }).split('\n')
    .filter(line => line.includes(directory)).map(line => {
      const [pid, ppid, pgid] = line.trim().split(/\s+/).map(Number);
      return { pid, ppid, pgid };
    });
}
for (const scenario of ['failure', 'SIGTERM', 'SIGINT']) {
  test(`composed held native and worker descendants are removed after ${scenario}`, { timeout: 180_000 }, async () => {
    const directory = mkdtempSync(join(tmpdir(), 'Task9-cleanup-test-'));
    const child = spawn(process.execPath, [script.pathname, '--fake'], {
      env: { ...process.env, TMPDIR: directory, TASK9_CARGO_TARGET_DIR: join(tmpdir(), 'Task9-codex-conversation-target'),
        ...(scenario === 'failure' ? { TASK9_INJECT_FAILURE: 'hold_thread_start' } : {}) },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let output = '';
    child.stdout.on('data', chunk => { output += chunk; });
    child.stderr.on('data', chunk => { output += chunk; });
    const ended = new Promise(resolve => child.once('exit', (code, signal) => resolve({ code, signal })));
    let observedHold = false;
    try {
      if (scenario !== 'failure') {
        for (let attempt = 0; attempt < 2400; attempt++) {
          const state = readdirSync(directory).find(name => name.startsWith('Task9-codex-conversation-'));
          if (state && existsSync(join(directory, state, 'native-requests.jsonl.hold_thread_start.stage'))) { observedHold = true; break; }
          if (child.exitCode !== null) break;
          await delay(50);
        }
        assert.equal(observedHold, true, output);
        child.kill(scenario);
      }
      const outcome = await ended;
      assert.notEqual(outcome.code, 0, 'failure injection must make the composed gate fail');
      if (scenario === 'failure') assert.match(output, /Task9 injected boundary assertion failure/);
      else assert.equal(outcome.code, scenario === 'SIGTERM' ? 143 : 130, output);
      assert.deepEqual(ownedProcesses(directory), [], 'runner left an owned descendant alive');
      assert.equal(execFileSync('podman', ['ps', '-a', '--filter', `name=^Task9-postgres-${child.pid}$`, '--format', '{{.Names}}'], { encoding: 'utf8' }).trim(), '');
      assert.deepEqual(readdirSync(directory).filter(name => name.startsWith('Task9-codex-conversation-')), [], 'fixture state removed only after resources are gone');
    } finally {
      if (process.env.TASK9_TEST_EVIDENCE_DIR) writeFileSync(join(process.env.TASK9_TEST_EVIDENCE_DIR, `cleanup-${scenario}.log`), output, { mode: 0o600 });
      if (child.exitCode === null && child.signalCode === null) { child.kill('SIGKILL'); await ended; }
      // Test failure cleanup is deliberately scoped to exact unique test directory.
      for (const { pid, pgid } of ownedProcesses(directory)) {
        try { process.kill(pid === pgid ? -pgid : pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; }
      }
      execFileSync('podman', ['rm', '-f', '--ignore', `Task9-postgres-${child.pid}`], { stdio: 'ignore' });
      rmSync(directory, { recursive: true, force: true });
    }
  });
}
