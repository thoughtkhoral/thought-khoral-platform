// SPDX-License-Identifier: Apache-2.0
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync } from 'node:fs';
const api = () => import('../lib/codex-schema-readiness.mjs');
const schema = () => JSON.parse(readFileSync(new URL('../fixtures/codex-conversation/schema-0006-0007.json', import.meta.url)));

test('actual schema check requires bounded authenticated read-only psql and returns only readiness', async () => {
  const { verifyConversationSchema } = await api();
  const calls = [];
  const result = verifyConversationSchema('task9-isolated-postgres', (program, argv, options) => {
    calls.push({ program, argv, options });
    return JSON.stringify(schema());
  });
  assert.deepEqual(result, { conversationSchemaReady: true, migrations: ['0006', '0007'] });
  assert.equal(calls.length, 1);
  const { program, argv, options } = calls[0];
  assert.equal(program, 'podman');
  assert.deepEqual(argv.slice(0, 2), ['exec', 'task9-isolated-postgres']);
  assert.match(argv[5], /PGPASSWORD="\$POSTGRES_PASSWORD"/);
  assert.match(argv[5], /--host 127\.0\.0\.1/);
  assert.match(argv[5], /PGCONNECT_TIMEOUT=5/);
  assert.match(argv[5], /statement_timeout=5000/);
  assert.match(argv.at(-1), /^BEGIN READ ONLY;/);
  assert.equal(options.timeout, 15000);
  assert.equal(options.maxBuffer, 131072);
});

test('missing tables, columns, valid constraints or active uniqueness fail closed', async () => {
  const { verifyConversationSchema } = await api();
  for (const change of [
    value => { delete value.agent_conversations; },
    value => { delete value.conversation_update_requests; },
    value => { delete value.conversation_tasks.columns.result; },
    value => { value.conversation_tasks.columns.last_update_ordinal.nullable = true; },
    value => { value.conversation_tasks.columns.last_update_ordinal.type = 'text'; },
    value => { value.conversation_tasks.columns.last_update_ordinal.default = '1'; },
    value => { value.conversation_tasks.constraints.pop(); },
    value => { value.agent_conversations.indexes = []; },
    value => { value.agent_conversations.indexes[0].valid = false; },
    value => { value.conversation_tasks.constraints[0].definition = 'CHECK (true)'; },
    value => { value.conversation_tasks.constraints[0].validated = false; },
    value => { value.room_requests.constraints = []; },
    value => { value.room_events.constraints = []; },
  ]) {
    const value = schema(); change(value);
    assert.throws(() => verifyConversationSchema('task9-postgres', () => JSON.stringify(value)), /schema readiness failed/);
  }
});

test('invalid targets and engine/output failures never disclose private diagnostics', async () => {
  const { verifyConversationSchema } = await api();
  let calls = 0;
  assert.throws(() => verifyConversationSchema('bad;container', () => { calls++; }), /invalid database container/);
  assert.equal(calls, 0);
  for (const execute of [() => { throw Error('private-secret'); }, () => 'private-secret', () => '{}', () => 'x'.repeat(131073)]) {
    assert.throws(() => verifyConversationSchema('task9-postgres', execute), error =>
      error.message === 'conversation database schema readiness failed');
  }
});
