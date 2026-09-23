# Agent Egress Fail-Stop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ensure both local Compose agents stop when the trusted egress sidecar exits, so they cannot continue operating with a stale kernel allowlist.

**Architecture:** Give the egress service a stable container name and make the gateway and reference agent join that container's PID namespace in addition to its existing network namespace. Linux terminates namespace members when its PID 1 exits. Prove the behavior by stopping only the egress owner in the local stack, observing both agents stop, and recreating the group.

**Tech Stack:** Podman 6.0.2, Podman Compose 1.6.0, Compose YAML, Python 3 `unittest` with PyYAML, POSIX shell.

**Spec:** `docs/superpowers/specs/2026-09-23-agent-egress-fail-stop-design.md`

## Global Constraints

- Limit changes to the local Compose deployment and its tests; do not alter the Kubernetes init-container and CNI NetworkPolicy model.
- Keep `network_mode: service:thought-khoral-agent-egress` for both agents and the existing health-gated startup dependencies.
- Only the egress service retains `NET_ADMIN`; gateway UID stays `10001:10001`, reference-agent UID stays `10002:10002`, and both retain `cap_drop: [ALL]` and read-only roots.
- Use explicit egress `container_name: thought-khoral-agent-egress` and `pid: container:thought-khoral-agent-egress`, because installed Podman Compose 1.6.0 forwards `pid` without translating `service:`.
- The Compose stack remains single-instance. Do not expose a container-runtime socket or put workload secrets in command-line arguments.
- Perform fault injection only against the local development stack. Recreate the stopped group before ending the task.

## File map

- Modify `compose.yaml`: stable egress container name and two agent PID namespace references.
- Modify `scripts/test-agent-egress.py`: static regression test for the Compose isolation contract.
- No Rust, Kubernetes, or image-source changes are expected.

---

### Task 1: Bind the Compose agents to the egress PID namespace

**Files:**
- Modify: `scripts/test-agent-egress.py` (`AgentEgressTests`, next to `test_compose_agents_cannot_join_the_general_network`)
- Modify: `compose.yaml` (egress and two agent service definitions)

**Interfaces:**
- Consumes: the existing `thought-khoral-agent-egress` Compose service and its `--hold` PID 1 entrypoint.
- Produces: egress container name `thought-khoral-agent-egress`; each agent's `pid` value `container:thought-khoral-agent-egress`.

- [ ] **Step 1: Write the failing configuration test**

Add this method to `AgentEgressTests` in `scripts/test-agent-egress.py`:

```python
    def test_compose_agents_die_with_the_egress_pid_namespace_owner(self):
        import yaml
        config = yaml.safe_load((ROOT / "compose.yaml").read_text())
        services = config["services"]
        egress = services["thought-khoral-agent-egress"]
        self.assertEqual(egress.get("container_name"), "thought-khoral-agent-egress")
        self.assertEqual(egress.get("cap_add"), ["NET_ADMIN"])
        for name, uid in [("thought-khoral-agent-gateway", 10001),
                          ("thought-khoral-reference-agent", 10002)]:
            service = services[name]
            self.assertEqual(service.get("pid"), "container:thought-khoral-agent-egress")
            self.assertEqual(service.get("network_mode"), "service:thought-khoral-agent-egress")
            self.assertEqual(service.get("user"), f"{uid}:{uid}")
            self.assertEqual(service.get("cap_drop"), ["ALL"])
            self.assertTrue(service.get("read_only"))
            self.assertIn("no-new-privileges:true", service.get("security_opt", []))
```

- [ ] **Step 2: Observe the test fail for the missing binding**

Run from `thought-khoral-platform`:

```sh
env PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=/private/tmp/thought-khoral-pyyaml-20260923 python3 scripts/test-agent-egress.py
```

Expected: the new test fails because `container_name` is absent; the existing egress tests still pass.

- [ ] **Step 3: Add the minimal Compose binding**

In `compose.yaml`, add the following exact fields without changing existing health dependencies or security settings:

```yaml
  thought-khoral-agent-egress:
    container_name: thought-khoral-agent-egress
```

```yaml
  thought-khoral-reference-agent:
    pid: container:thought-khoral-agent-egress
```

```yaml
  thought-khoral-agent-gateway:
    pid: container:thought-khoral-agent-egress
```

- [ ] **Step 4: Observe the test pass and inspect the rendered model**

```sh
env PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=/private/tmp/thought-khoral-pyyaml-20260923 python3 scripts/test-agent-egress.py
podman-compose -f compose.yaml config
```

Expected: all Python tests pass; rendered Compose retains the explicit name, both `pid` values, and both `network_mode` values.

- [ ] **Step 5: Recreate and verify namespace membership**

Stop the local development stack first to avoid Keycloak memory pressure, then recreate from the current images:

```sh
podman-compose -f compose.yaml stop
podman-compose -f compose.yaml up -d --no-build --force-recreate
podman inspect --format '{{.HostConfig.PidMode}}' thought-khoral_thought-khoral-agent-gateway_1
podman inspect --format '{{.HostConfig.PidMode}}' thought-khoral_thought-khoral-reference-agent_1
podman exec thought-khoral-agent-egress readlink /proc/self/ns/pid
podman exec thought-khoral_thought-khoral-agent-gateway_1 readlink /proc/self/ns/pid
podman exec thought-khoral_thought-khoral-reference-agent_1 readlink /proc/self/ns/pid
bash scripts/smoke.sh
```

Expected: both PID modes reference `thought-khoral-agent-egress`; all three PID namespace links match; full smoke passes. If Podman Compose rejects the PID mode or the links differ, stop and revise the design rather than treating static configuration as sufficient.

- [ ] **Step 6: Prove fail-stop under a controlled sidecar exit**

Only after Step 5 succeeds, stop the exact local egress owner and inspect the agents:

```sh
podman kill --signal KILL thought-khoral-agent-egress
podman inspect --format '{{.State.Status}}' thought-khoral-agent-egress
podman inspect --format '{{.State.Status}}' thought-khoral_thought-khoral-agent-gateway_1
podman inspect --format '{{.State.Status}}' thought-khoral_thought-khoral-reference-agent_1
```

Expected: egress and both agents are not `running` after the engine observes the exit. Account for the gateway's `on-failure:3` restart attempts; it must not resume in an unguarded namespace. If either agent remains running, this task has failed and must not be committed as a fail-stop guarantee.

- [ ] **Step 7: Restore the local stack and rerun the task path**

```sh
podman-compose -f compose.yaml up -d --no-build --force-recreate
bash scripts/smoke-agent-egress.sh
node scripts/smoke-agent-gateway.mjs
```

Expected: all services recover; UID isolation and both A2A skills pass after recreation. If recovery fails, diagnose and restore the stack before ending the task.

- [ ] **Step 8: Verify the diff and commit the tested change**

```sh
git diff --check
git status --short
git add compose.yaml scripts/test-agent-egress.py
git diff --cached --check
git commit -m 'Stop agents when egress namespace owner exits'
git status --short
```

Expected: no whitespace errors, only the two intended files are committed, and the worktree is clean. Report the commit and both the normal-smoke and fault-injection evidence.
