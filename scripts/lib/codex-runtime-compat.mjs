// SPDX-License-Identifier: Apache-2.0
import { validatePackagedProfile } from './codex-packaged-profile.mjs';
import { verifyConversationSchema } from './codex-schema-readiness.mjs';

export function verifyPackagedSchema(profile, packet, execute) {
  validatePackagedProfile(profile);
  if (profile.id !== 'packaged-v1.1-runtime-compat') return null;
  return verifyConversationSchema(packet.postgresContainer, execute);
}
