#!/usr/bin/env python3
"""Configuration regressions for the agent network boundary (requires PyYAML)."""
import pathlib
import os
import shlex
import subprocess
import tempfile
import unittest
import yaml

ROOT = pathlib.Path(__file__).resolve().parents[1]

class AgentEgressTests(unittest.TestCase):
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
            rules = {"iptables": [], "ip6tables": []}
            attached = set()
            for line in log.read_text().splitlines():
                command, *args = shlex.split(line)
                if "OUTPUT" in args:
                    attached.add(command)
                if "THOUGHT_AGENT_EGRESS" not in args or "-j" not in args or "OUTPUT" in args:
                    continue
                if "-I" in args:
                    rules[command].insert(0, args)
                else:
                    rules[command].append(args)
            self.assertEqual(attached, {"iptables", "ip6tables"})

            def permits(command, uid, address, port, protocol="tcp", interface="eth0"):
                for rule in rules[command]:
                    values = {flag: rule[rule.index(flag) + 1] for flag in
                              ["-o", "--ctstate", "--uid-owner", "-p", "--ctorigdst", "--ctorigdstport", "-j"] if flag in rule}
                    if "--ctstate" in values:  # All probes initiate NEW connections.
                        continue
                    if any(values.get(flag, str(value)) != str(value) for flag, value in
                           [("-o", interface), ("--uid-owner", uid), ("-p", protocol),
                            ("--ctorigdst", address), ("--ctorigdstport", port)]):
                        continue
                    return values["-j"] == "ACCEPT"
                self.fail("every connection must reach an explicit terminal policy")

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
