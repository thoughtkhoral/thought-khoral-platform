#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Trusted PID/network namespace owner. Exiting PID 1 kills its dependents."""
import pathlib
import socket
import subprocess
import time

CHAIN = 'THOUGHT_CODEX_EGRESS'

def rule(binary, *args):
    return subprocess.check_output([binary, '-w', *args], text=True)

def install(binary, dns):
    rule(binary, '-N', CHAIN)
    for uid, port in [('10003', '3128'), ('10003', '9091'), ('10004', '3128'), ('0', '3128')]:
        rule(binary, '-A', CHAIN, '-o', 'lo', '-m', 'owner', '--uid-owner', uid,
             '-p', 'tcp', '-m', 'conntrack', '--ctorigdstport', port, '--ctdir', 'ORIGINAL', '-j', 'ACCEPT')
    for uid, port in [('10004', '3128'), ('10003', '9091')]:
        rule(binary, '-A', CHAIN, '-m', 'owner', '--uid-owner', uid, '-p', 'tcp',
             '-m', 'conntrack', '--ctstate', 'ESTABLISHED', '--ctorigdstport', port, '--ctdir', 'REPLY', '-j', 'ACCEPT')
    if binary == 'iptables':
        # TCP teardown can emit a reply ACK without a full owning socket. Admit
        # only the observed timestamped, empty ACK on an existing loopback flow.
        # IPv4 length 52 = IHL 20 + TCP header 32: no payload or IP options.
        rule(binary, '-A', CHAIN, '-o', 'lo', '-s', '127.0.0.1/32', '-d', '127.0.0.1/32',
             '-p', 'tcp', '--tcp-flags', 'ALL', 'ACK', '-m', 'owner', '!', '--socket-exists',
             '-m', 'conntrack', '--ctstate', 'ESTABLISHED', '--ctdir', 'REPLY',
             '--ctorigdst', '127.0.0.1', '--ctorigdstport', '3128',
             '-m', 'length', '--length', '52', '-m', 'u32', '--u32',
             '0&0xFF000000=0x45000000&&32&0xFFFF0000=0x80100000', '-j', 'ACCEPT')
    for address in dns:
        for uid in ['0', '10004']:
            for protocol in ['tcp', 'udp']:
                rule(binary, '-A', CHAIN, '-m', 'owner', '--uid-owner', uid, '-p', protocol,
                     '-d', address, '--dport', '53', '-j', 'ACCEPT')
    # Only the proxy can initiate external TLS. Its closed CONNECT policy narrows host.
    rule(binary, '-A', CHAIN, '-m', 'owner', '--uid-owner', '10004', '-p', 'tcp', '--dport', '443', '-j', 'ACCEPT')
    rule(binary, '-A', CHAIN, '-j', 'REJECT')
    rule(binary, '-I', 'OUTPUT', '1', '-j', CHAIN)
    return rule(binary, '-S')

def proxy_ready():
    try:
        with socket.create_connection(('127.0.0.1', 3128), timeout=1) as stream:
            stream.sendall(b'GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n')
            return stream.recv(128).startswith(b'HTTP/1.1 200 ')
    except OSError:
        return False

def main():
    dns = [line.split()[1] for line in pathlib.Path('/etc/resolv.conf').read_text().splitlines() if line.startswith('nameserver ')]
    policies = {binary: install(binary, [x for x in dns if (':' in x) == ipv6])
                for binary, ipv6 in [('iptables', False), ('ip6tables', True)]}
    pathlib.Path('/run/codex-policy-ready').touch()
    deadline = time.monotonic() + 60
    observed_proxy = False
    while True:
        # Compare whole filter rules (no counters), including OUTPUT jump ordering.
        for binary, expected in policies.items():
            if rule(binary, '-S') != expected:
                raise RuntimeError('kernel egress policy changed')
        healthy = proxy_ready()
        if healthy:
            observed_proxy = True
        elif observed_proxy or time.monotonic() > deadline:
            raise RuntimeError('provider proxy unavailable')
        time.sleep(1)

if __name__ == '__main__':
    try:
        main()
    except Exception:
        pathlib.Path('/run/codex-policy-ready').unlink(missing_ok=True)
        raise SystemExit('Codex isolation lost; namespace stopping')
