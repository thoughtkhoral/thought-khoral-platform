# Agent egress fail-stop design

## Purpose and scope

The Compose deployment must stop both agent processes if the trusted egress
sidecar exits after it can no longer maintain the kernel allowlist. This closes
the operational gap where the agents can retain a shared network namespace
with stale firewall rules after the sidecar exits. The change is limited to the
local Compose deployment and its tests. Kubernetes continues to use its
one-shot egress init container plus CNI NetworkPolicy; that is a separate
runtime model and is not changed here.

## Runtime contract

The egress container is the PID namespace owner. Its entrypoint shell remains
PID 1 while `--hold` refreshes peer addresses. Both agent containers join the
egress container's network namespace, as they do today, and also join its PID
namespace. If the egress process exits for any reason, Linux terminates all
processes in that PID namespace. Neither agent may continue with a stale
allowlist, even if the container engine leaves its network namespace allocated.

The installed Podman Compose 1.6.0 forwards `pid` directly to Podman and does
not translate `pid: service:<name>` into a container reference. Therefore the
egress service gets the explicit container name `thought-khoral-agent-egress`,
and both agents use `pid: container:thought-khoral-agent-egress`. Keep the
existing `network_mode: service:thought-khoral-agent-egress` and startup health
dependencies. This Compose project is single-instance; the explicit name
prevents running a second copy concurrently under a different project name.

## Security and failure behavior

The sidecar alone retains `NET_ADMIN`. The gateway remains UID 10001 and the
reference agent UID 10002, both with all capabilities dropped and read-only
root filesystems. Sharing a PID namespace lets them see each other's process
metadata, but different UIDs prevent one agent from signaling or reading the
other's protected process data under the normal Linux permission model. Do not
put workload secrets in command-line arguments. The privileged sidecar is
already trusted to configure the shared network namespace.

An unrecoverable firewall-write error makes the sidecar exit; the PID-namespace
dependency then kills both agents. A transient refresh error that can revoke
the active peer allowance is handled by the existing in-process retry loop and
does not kill the agents. A sidecar crash or operator stop also stops both
agents. Recovery requires recreating the egress owner and its dependent agent
containers as a group; automatic restart of an individual agent must not let
it run in a newly unguarded namespace.

This does not claim that Linux can revoke a stale rule when all kernel policy
writes fail. It instead ensures that this deployment's agent processes do not
continue operating once the policy owner is gone. It does not defend against
an independently compromised process that deliberately bypasses the trusted
container entrypoint or container runtime.

## Verification

1. A configuration test checks the explicit egress name, the agents' shared
   PID mode, unchanged shared network mode, distinct UIDs, and capability drops.
2. `podman-compose config` and live `podman inspect` confirm the installed
   provider passes the intended PID mode and all three containers have the
   same PID namespace, separate from the other platform services.
3. The full local smoke suite passes with the new namespace topology.
4. A controlled stop of the egress PID 1 causes both agents to stop. The test
   checks the egress and agent container states and demonstrates that a task
   cannot be processed until the group is recreated. Then recreate the group
   and rerun the A2A and egress smoke tests.

The controlled stop runs only against the local development stack; it must
not be used as a production fault injection command.
