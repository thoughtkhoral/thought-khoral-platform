import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const bootstrapPath = fileURLToPath(
  new URL('../ui/thought-khoral-bootstrap.js', import.meta.url),
);
const source = readFileSync(bootstrapPath, 'utf8');

const requiredMarkers = [
  'thoughtKhoralWorkspace',
  'session.authenticate',
  'thought-khoral.oidc.tokens',
  "roomId: pageUrl.searchParams.get('room') ?? undefined",
  'onEnterRoom',
  'onLeaveRoom',
];
const missingMarkers = requiredMarkers.filter((marker) => !source.includes(marker));
if (missingMarkers.length > 0) {
  throw new Error(`bootstrap is missing required markers: ${missingMarkers.join(', ')}`);
}

if (source.includes('10000000-0000-4000-8000-000000000001')) {
  throw new Error('bootstrap contains the hardcoded retained-room fallback');
}

if (source.includes('room.join')) {
  throw new Error('bootstrap must not join a room during module evaluation');
}

console.log('bootstrap lifecycle invariants passed');
