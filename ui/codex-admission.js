// SPDX-License-Identifier: Apache-2.0
// Same closed host declaration consumed by the reviewed workspace UI.
export async function readConversationAdmission(request = fetch) {
  try {
    const response = await request('/codex-admission/admission.json', {
      cache: 'no-store', signal: AbortSignal.timeout(2000),
    });
    if (!response.ok) return undefined;
    const text = await response.text();
    if (text.length > 2048) return undefined;
    const envelope = JSON.parse(text);
    if (!envelope || Object.keys(envelope).sort().join(',') !== 'admission,expiresAtUnixMs') return undefined;
    const remaining = envelope.expiresAtUnixMs - Date.now();
    if (!Number.isSafeInteger(envelope.expiresAtUnixMs) || remaining <= 0 || remaining > 5000) return undefined;
    const value = envelope.admission;
    if (!value || typeof value !== 'object' || Array.isArray(value)) return undefined;
    const required = {
      profileVersion: 'thought-khoral.agent-conversation.v1',
      agentId: '74686f75-6768-746b-686f-72616c000004',
      conversationScope: 'room', invocation: 'explicitly-addressed',
      delivery: 'room', roomHistory: 'baseline-and-delta',
    };
    const optional = ['modelSelection', 'reasoningEffort', 'usageReporting'];
    if (Object.entries(required).some(([key, expected]) => value[key] !== expected)) return undefined;
    if (Object.keys(value).some(key => !Object.hasOwn(required, key) && !optional.includes(key))) return undefined;
    if (optional.some(key => key in value && typeof value[key] !== 'boolean')) return undefined;
    return value;
  } catch { return undefined; }
}
