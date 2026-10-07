#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail
health() {
  local response secret
  secret=$(cat /run/secrets/codex-invocation)
  exec 3<>/dev/tcp/127.0.0.1/9091
  printf 'GET /.well-known/agent-card.json HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer %s\r\nConnection: close\r\n\r\n' "$secret" >&3
  IFS= read -r response <&3
  [[ "$response" == 'HTTP/1.1 200 '* ]]
  exec 3<&-
}
if [[ ${1:-} == --health ]]; then health; exit; fi
[[ $(id -u) == 10003 && $(id -g) == 10003 ]]
# Remove previous readiness before every potentially failing startup check.
rm -f /run/codex-admission/admission.json
[[ ${THOUGHT_KHORAL_CODEX_IMAGE:-} =~ ^(sha256:[0-9a-f]{64}|[^[:space:]]+@sha256:[0-9a-f]{64})$ ]]
for directory in native receipts; do
  path=/var/lib/thought-khoral-codex/$directory
  [[ -d "$path" && ! -L "$path" && $(stat -c '%u:%g:%a' "$path") == '10003:10003:700' ]]
  probe=$(mktemp "$path/.write-check.XXXXXXXX")
  rm "$probe"
done
[[ ! -e /var/lib/thought-khoral-codex/native/config.toml && ! -e /var/lib/thought-khoral-codex/native/auth.json ]]
[[ $(stat -c '%u:%g:%a' /opt/thought-khoral-codex/workspace) == '0:0:555' ]]
[[ -d /run/codex-admission && ! -L /run/codex-admission && $(stat -c '%u:%g:%a' /run/codex-admission) == '10003:10003:755' ]]
for file in /run/secrets/codex-provider /run/secrets/codex-invocation; do
  [[ -f "$file" && -r "$file" && $(wc -c < "$file") -le 4096 ]]
  value=$(cat "$file")
  [[ ${#value} -ge 16 && "$value" != *[$'\001'-$'\040'$'\177']* ]]
done
[[ $(cat /run/secrets/codex-provider) != "$(cat /run/secrets/codex-invocation)" ]]
verification=$(/usr/local/bin/thought-khoral-codex-agent --verify-package)
grep -Fxq 'worker package verified; codex-cli 0.160.0; uid/gid 10003; no inference' <<< "$verification"
# The existing version-only Task 6 image is deliberately insufficient.
grep -Fxq 'tool policy verified; exposed tools: []' <<< "$verification"
[[ ${THOUGHT_KHORAL_CODEX_ISOLATION_VERIFIED:-} == 1 ]]
if [[ ${1:-} == --check ]]; then printf 'packaging verified\n'; exit; fi
[[ $# == 0 ]]
umask 077
rm -f /run/codex-admission/admission.json
/usr/local/bin/thought-khoral-codex-agent &
worker=$!
cleanup() { rm -f /run/codex-admission/admission.json; kill "$worker" 2>/dev/null || true; }
trap cleanup EXIT TERM INT
for ((attempt=0; attempt<100; attempt++)); do
  kill -0 "$worker"
  if health 2>/dev/null; then break; fi
  sleep 0.1
done
health
while kill -0 "$worker" 2>/dev/null; do
  timeout 2 /bin/bash /opt/platform/codex-start.sh --health
  expires_ms=$(( ( $(date +%s) + 5 ) * 1000 ))
  printf '{"expiresAtUnixMs":%s,"admission":' "$expires_ms" > /run/codex-admission/admission.json.tmp
  cat >> /run/codex-admission/admission.json.tmp <<'JSON'
{"profileVersion":"thought-khoral.agent-conversation.v1","agentId":"74686f75-6768-746b-686f-72616c000004","conversationScope":"room","invocation":"explicitly-addressed","delivery":"room","roomHistory":"baseline-and-delta","modelSelection":true,"reasoningEffort":true,"usageReporting":true}}
JSON
  chmod 644 /run/codex-admission/admission.json.tmp
  mv /run/codex-admission/admission.json.tmp /run/codex-admission/admission.json
  sleep 1
done
wait "$worker"
