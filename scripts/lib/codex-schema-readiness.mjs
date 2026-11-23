// SPDX-License-Identifier: Apache-2.0
import { readFileSync } from 'node:fs';
import { isDeepStrictEqual } from 'node:util';
import { execFileSync } from 'node:child_process';

// Catalog metadata only; no room/task contents or migration-ledger assumptions.
const query = readFileSync(new URL('./codex-schema-readiness.sql', import.meta.url), 'utf8');
const expected = JSON.parse(readFileSync(new URL('../fixtures/codex-conversation/schema-0006-0007.json', import.meta.url), 'utf8'));
const psql = 'export PGPASSWORD="$POSTGRES_PASSWORD" PGCONNECT_TIMEOUT=5 '
  + 'PGOPTIONS="-c default_transaction_read_only=on -c statement_timeout=5000 -c lock_timeout=1000"; '
  + 'exec psql -X -qAt --no-password --host 127.0.0.1 --username "$POSTGRES_USER" '
  + '--dbname "$POSTGRES_DB" --set ON_ERROR_STOP=1 --command "$1"';

export function verifyConversationSchema(container, execute = (program, argv, options) =>
  execFileSync(program, argv, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], ...options })) {
  if (typeof container !== 'string' || !/^[A-Za-z0-9][A-Za-z0-9_.-]{0,127}$/.test(container))
    throw new Error('invalid database container');
  try {
    const raw = execute('podman', ['exec', container, '/bin/sh', '-eu', '-c', psql, 'schema-readiness', query],
      { timeout: 15000, maxBuffer: 131072 });
    if (typeof raw !== 'string' || Buffer.byteLength(raw) > 131072) throw new Error();
    const actual = JSON.parse(raw);
    for (const [name, required] of Object.entries(expected)) {
      const table = actual[name];
      if (!table || !isDeepStrictEqual(table.columns, required.columns)
        || !isDeepStrictEqual(table.constraints, required.constraints)
        || !isDeepStrictEqual(table.indexes, required.indexes)) throw new Error();
    }
    return { conversationSchemaReady: true, migrations: ['0006', '0007'] };
  } catch {
    // Never retain SQL output, local credentials, database errors or room data.
    throw new Error('conversation database schema readiness failed');
  }
}
