#!/usr/bin/env python3
"""Configuration regressions for the agent network boundary (requires PyYAML)."""
import pathlib
import os
import shlex
import subprocess
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]

class AgentEgressTests(unittest.TestCase):
    def test_refresh_rule_failure_clears_active_peer_allowlist(self):
        with tempfile.TemporaryDirectory() as directory:
            fixture = pathlib.Path(directory)
            log = fixture / "rules"
            iptables = fixture / "iptables"
            iptables.write_text('''#!/bin/sh
printf "iptables %s\\n" "$*" >> "$RULE_LOG"
case "$*" in
  *"-A THOUGHT_AGENT_PEERS_B "*) exit 1;;
esac
''')
            iptables.chmod(0o755)
            ip6tables = fixture / "ip6tables"
            ip6tables.write_text('#!/bin/sh\nprintf "ip6tables %s\\n" "$*" >> "$RULE_LOG"\n')
            ip6tables.chmod(0o755)
            lookup = fixture / "getent"
            lookup.write_text('''#!/bin/sh
case "$2" in
  thought-khoral-room-gateway) echo "10.20.0.2 STREAM room";;
  thought-khoral-keycloak)
    if [ -f "$LOOKUP_CHANGED" ]; then
      echo "10.20.0.5 STREAM keycloak"
    else
      touch "$LOOKUP_CHANGED"
      echo "10.20.0.3 STREAM keycloak"
    fi;;
esac
''')
            lookup.chmod(0o755)
            sleep = fixture / "sleep"
            sleep.write_text('#!/bin/sh\n[ ! -f "$SLEEP_ONCE" ] || exit 1\ntouch "$SLEEP_ONCE"\n')
            sleep.chmod(0o755)
            result = subprocess.run(
                ["sh", str(ROOT / "scripts/agent-egress.sh"), "--hold"],
                timeout=10,
                env={**os.environ, "PATH": str(fixture) + ":" + os.environ["PATH"],
                     "RULE_LOG": str(log), "LOOKUP_CHANGED": str(fixture / "lookup-changed"),
                     "SLEEP_ONCE": str(fixture / "sleep-once")},
            )
            self.assertNotEqual(result.returncode, 0)
            rules = log.read_text().splitlines()
            failed_append = next(i for i, rule in enumerate(rules)
                                 if "-A THOUGHT_AGENT_PEERS_B " in rule)
            self.assertIn("iptables -w -F THOUGHT_AGENT_PEERS_A", rules[failed_append + 1:])

    def test_hold_mode_replaces_changed_peer_addresses_and_fails_closed(self):
        # Kernel operations are recorded, while the real policy script handles
        # startup, two DNS refreshes, and the active-chain switches.
        with tempfile.TemporaryDirectory() as directory:
            fixture = pathlib.Path(directory)
            log = fixture / "rules"
            for name in ["iptables", "ip6tables"]:
                tool = fixture / name
                tool.write_text('#!/bin/sh\nprintf "%s %s\\n" "' + name + '" "$*" >> "$RULE_LOG"\n')
                tool.chmod(0o755)
            lookup = fixture / "getent"
            lookup.write_text('''#!/bin/sh
count=$(cat "$DNS_COUNT")
echo $((count + 1)) > "$DNS_COUNT"
case "$count:$2" in
  0:thought-khoral-room-gateway) echo "10.20.0.2 STREAM room";;
  1:thought-khoral-keycloak) echo "10.20.0.3 STREAM keycloak";;
  2:thought-khoral-room-gateway) echo "10.20.0.4 STREAM room";;
  3:thought-khoral-keycloak) echo "10.20.0.5 STREAM keycloak";;
  4:thought-khoral-room-gateway) echo "10.20.0.2 STREAM stale"; exit 1;;
  5:thought-khoral-keycloak) echo "10.20.0.3 STREAM stale";;
  *) exit 1;;
esac
''')
            lookup.chmod(0o755)
            sleep = fixture / "sleep"
            sleep.write_text('''#!/bin/sh
count=$(cat "$SLEEP_COUNT")
echo $((count + 1)) > "$SLEEP_COUNT"
[ "$count" -lt 2 ]
''')
            sleep.chmod(0o755)
            (fixture / "dns-count").write_text("0")
            (fixture / "sleep-count").write_text("0")
            result = subprocess.run(
                ["sh", str(ROOT / "scripts/agent-egress.sh"), "--hold"],
                timeout=10,
                env={**os.environ, "PATH": str(fixture) + ":" + os.environ["PATH"],
                     "RULE_LOG": str(log), "DNS_COUNT": str(fixture / "dns-count"),
                     "SLEEP_COUNT": str(fixture / "sleep-count")},
            )
            self.assertNotEqual(result.returncode, 0, "test clock must stop the hold loop")

            chains = {}
            active = None
            snapshots = []
            for line in log.read_text().splitlines():
                command, *args = shlex.split(line)
                if command != "iptables":
                    continue
                if "-N" in args:
                    chains[args[args.index("-N") + 1]] = []
                elif "-F" in args:
                    chains[args[args.index("-F") + 1]] = []
                elif "-A" in args:
                    chains[args[args.index("-A") + 1]].append(args)
                elif "-R" in args:
                    chain = args[args.index("-R") + 1]
                    if chain == "THOUGHT_AGENT_EGRESS":
                        active = args[args.index("-j") + 1]
                        snapshots.append({rule[rule.index("--ctorigdst") + 1]
                                          for rule in chains[active] if "--ctorigdst" in rule})
            self.assertEqual(snapshots, [
                {"10.20.0.4", "10.20.0.5"},
                set(),
            ])

    def test_partial_failed_startup_lookup_never_installs_peer_allowances(self):
        with tempfile.TemporaryDirectory() as directory:
            fixture = pathlib.Path(directory)
            log = fixture / "rules"
            for name in ["iptables", "ip6tables"]:
                tool = fixture / name
                tool.write_text('#!/bin/sh\nprintf "%s %s\\n" "' + name + '" "$*" >> "$RULE_LOG"\n')
                tool.chmod(0o755)
            lookup = fixture / "getent"
            lookup.write_text('''#!/bin/sh
case "$2" in
  thought-khoral-room-gateway) echo "10.20.0.2 STREAM partial"; exit 1;;
  thought-khoral-keycloak) echo "10.20.0.3 STREAM keycloak";;
esac
''')
            lookup.chmod(0o755)
            result = subprocess.run(
                ["sh", str(ROOT / "scripts/agent-egress.sh")],
                env={**os.environ, "PATH": str(fixture) + ":" + os.environ["PATH"],
                     "RULE_LOG": str(log)},
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(log.exists(), "no peer rule may be installed after failed DNS")

    def test_kernel_setup_emits_rules_that_deny_unregistered_destinations(self):
        # Run the real setup script; replace only privileged kernel commands
        # and DNS lookup, then evaluate the emitted NEW-connection rules.
        with tempfile.TemporaryDirectory() as directory:
            fixture = pathlib.Path(directory)
            log = fixture / "rules"
            for name in ["iptables", "ip6tables"]:
                tool = fixture / name
                tool.write_text('#!/bin/sh\nprintf "%s %s\\n" "' + name + '" "$*" >> "$RULE_LOG"\n')
                tool.chmod(0o755)
            lookup = fixture / "getent"
            lookup.write_text('#!/bin/sh\ncase "$2" in\n thought-khoral-room-gateway) echo "10.20.0.2 STREAM room";;\n thought-khoral-keycloak) echo "10.20.0.3 STREAM keycloak";;\n *) exit 1;;\nesac\n')
            lookup.chmod(0o755)
            subprocess.run(["sh", str(ROOT / "scripts/agent-egress.sh")], check=True,
                           env={**os.environ, "PATH": str(fixture) + ":" + os.environ["PATH"], "RULE_LOG": str(log)})
            rules = {}
            attached = set()
            for line in log.read_text().splitlines():
                command, *args = shlex.split(line)
                if "OUTPUT" in args:
                    attached.add(command)
                if "-j" not in args or "OUTPUT" in args:
                    continue
                action = "-I" if "-I" in args else "-A"
                chain = args[args.index(action) + 1]
                key = (command, chain)
                rules.setdefault(key, [])
                if action == "-I":
                    rules[key].insert(0, args)
                else:
                    rules[key].append(args)
            self.assertEqual(attached, {"iptables", "ip6tables"})

            def permits(command, uid, address, port, protocol="tcp", interface="eth0"):
                def evaluate(chain):
                    for rule in rules.get((command, chain), []):
                        values = {flag: rule[rule.index(flag) + 1] for flag in
                                  ["-o", "--ctstate", "--uid-owner", "-p", "--ctorigdst", "--ctorigdstport", "-j"] if flag in rule}
                        if any(values.get(flag, str(value)) != str(value) for flag, value in
                               [("-o", interface), ("--uid-owner", uid), ("-p", protocol),
                                ("--ctorigdst", address), ("--ctorigdstport", port)]):
                            continue
                        target = values["-j"]
                        if target in ["ACCEPT", "REJECT"]:
                            return target == "ACCEPT"
                        nested = evaluate(target)
                        if nested is not None:
                            return nested
                    return None
                decision = evaluate("THOUGHT_AGENT_EGRESS")
                self.assertIsNotNone(decision, "every connection must reach a terminal policy")
                return decision

            self.assertTrue(permits("iptables", 10001, "10.20.0.2", 8080))
            self.assertTrue(permits("iptables", 10001, "10.20.0.3", 8080))
            for uid in [10001, 10002]:
                self.assertFalse(permits("iptables", uid, "1.1.1.1", 443))
                self.assertFalse(permits("iptables", uid, "10.20.0.2", 5432))
                self.assertFalse(permits("ip6tables", uid, "2001:db8::1", 443))
                self.assertTrue(permits("iptables", uid, "127.0.0.1", 9090, interface="lo"))
            self.assertFalse(permits("iptables", 10002, "10.20.0.2", 8080))
            self.assertFalse(permits("iptables", 10002, "10.20.0.3", 8080))

    def test_compose_agents_cannot_join_the_general_network(self):
        import yaml
        config = yaml.safe_load((ROOT / "compose.yaml").read_text())
        services = config["services"]
        for name in ["thought-khoral-agent-gateway", "thought-khoral-reference-agent"]:
            self.assertEqual(services[name].get("network_mode"), "service:thought-khoral-agent-egress")
            self.assertNotIn("networks", services[name])
            self.assertEqual(services[name].get("cap_drop"), ["ALL"])
        self.assertTrue(config["networks"]["thought-khoral-agent-internal"]["internal"])
        peers = {name for name, service in services.items() if "thought-khoral-agent-internal" in service.get("networks", [])}
        self.assertEqual(peers, {"thought-khoral-agent-egress", "thought-khoral-room-gateway", "thought-khoral-keycloak"})

    def test_kubernetes_egress_requires_both_kernel_filter_and_network_policy(self):
        import yaml
        deployment = next(yaml.safe_load_all((ROOT / "kube/agent-gateway.yaml").read_text()))
        pod = deployment["spec"]["template"]["spec"]
        self.assertEqual(pod.get("initContainers", [{}])[0].get("name"), "agent-egress")
        self.assertEqual(pod["initContainers"][0]["securityContext"]["capabilities"], {"drop": ["ALL"], "add": ["NET_ADMIN"]})
        for container in pod["containers"]:
            security = container["securityContext"]
            expected_uid = 10002 if container["name"] == "thought-khoral-reference-agent" else 10001
            self.assertEqual(security.get("runAsUser", pod["securityContext"]["runAsUser"]), expected_uid)
            self.assertEqual(security["capabilities"], {"drop": ["ALL"]})
        policy_path = ROOT / "kube/agent-egress-policy.yaml"
        self.assertTrue(policy_path.exists(), "CNI egress policy must be deployed")
        policy = yaml.safe_load(policy_path.read_text())
        self.assertEqual(policy["spec"]["policyTypes"], ["Egress"])
        for rule in policy["spec"]["egress"]:
            self.assertTrue(rule.get("to"))
            self.assertTrue(rule.get("ports"))
            self.assertFalse(any(peer.get("ipBlock", {}).get("cidr") == "0.0.0.0/0" for peer in rule["to"]))

if __name__ == "__main__":
    unittest.main()
