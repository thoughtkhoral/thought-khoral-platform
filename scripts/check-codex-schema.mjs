#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
import { verifyConversationSchema } from './lib/codex-schema-readiness.mjs';
try {
  if (process.argv.length !== 3) throw new Error('usage: check-codex-schema.mjs <owned-postgres-container>');
  console.log(JSON.stringify(verifyConversationSchema(process.argv[2])));
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
