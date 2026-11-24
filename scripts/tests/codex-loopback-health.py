#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Owned real-kernel teardown regression; no provider keys, requests or ports."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time
import uuid

OWNER = r'''
import importlib.util,os,pathlib,subprocess,time
spec=importlib.util.spec_from_file_location('egress',os.environ.get('FIXTURE_POLICY','/opt/codex/codex-egress.py'))
mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
mod.install('iptables',[]);mod.install('ip6tables',[])
common=['-o','lo','-s','127.0.0.1/32','-d','127.0.0.1/32','-p','tcp','--tcp-flags','ALL','ACK']
categories={
 'socketless-established-reply':['-m','owner','!','--socket-exists','-m','conntrack','--ctstate','ESTABLISHED','--ctdir','REPLY','--ctorigdstport','3128'],
 'eligible-zero-payload':['-m','owner','!','--socket-exists','-m','conntrack','--ctstate','ESTABLISHED','--ctdir','REPLY','--ctorigdstport','3128','-m','length','--length','52','-m','u32','--u32','0&0xFF000000=0x45000000&&32&0xFFFF0000=0x80100000'],
 'header-shape':['-m','length','--length','52','-m','u32','--u32','0&0xFF000000=0x45000000&&32&0xFFFF0000=0x80100000'],
 'socketless-established-original':['-m','owner','!','--socket-exists','-m','conntrack','--ctstate','ESTABLISHED','--ctdir','ORIGINAL','--ctorigdstport','3128'],
 'socketless-invalid':['-m','owner','!','--socket-exists','-m','conntrack','--ctstate','INVALID'],
 'socketless-untracked':['-m','owner','!','--socket-exists','-m','conntrack','--ctstate','UNTRACKED'],
 'socketless-all':['-m','owner','!','--socket-exists'],
 'owned-established-reply':['-m','owner','--socket-exists','-m','conntrack','--ctstate','ESTABLISHED','--ctdir','REPLY','--ctorigdstport','3128'],
}
# No target: these rules only count; every packet keeps the original verdict.
for name,parts in categories.items():
 mod.rule('iptables','-I',mod.CHAIN,'1',*common,*parts,'-m','comment','--comment','task9-'+name)
pathlib.Path('/run/fixture-ready').touch()
while True:time.sleep(1)
'''

SNAPSHOT = r'''
import collections,json,pathlib,re,subprocess
raw=subprocess.check_output(['iptables-save','-c','-t','filter'],text=True)
counts={}
for line in raw.splitlines():
 match=re.match(r'\[(\d+):\d+\].*--comment "?task9-([a-z-]+)"?',line)
 if match:counts[match[2]]=int(match[1])
rows=[line.split() for line in pathlib.Path('/proc/net/tcp').read_text().splitlines()[1:]]
print(json.dumps({'counters':counts,'clientLastAck':sum(row[3]=='09' and row[2].endswith(':0C38') for row in rows),'serverTimeWait':sum(row[3]=='06' and row[1].endswith(':0C38') for row in rows)}))
'''

HEALTH = r'''
import socket
for _ in range(3):
 with socket.create_connection(('127.0.0.1',3128),timeout=1) as stream:
  stream.sendall(b'GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n')
  response=b''
  while True:
   part=stream.recv(128)
   if not part:break
   response+=part
   assert len(response)<=1024
  assert response.startswith(b'HTTP/1.1 200 ')
'''

RAW_PACKET = r'''
import json,socket,struct,sys
case=json.loads(sys.argv[1]);ip_options=bytes([1])*case['ipOptions'];tcp_options=bytes([1])*case['tcpOptions'];payload=b'x'*case['payload']
def checksum(data):
 if len(data)%2:data+=b'\0'
 total=sum(struct.unpack('!'+str(len(data)//2)+'H',data))
 while total>>16:total=(total&65535)+(total>>16)
 return (~total)&65535
address=socket.inet_aton('127.0.0.1');tcp_length=20+len(tcp_options)
offset=((tcp_length//4)<<4)|case.get('reserved',0)
tcp=struct.pack('!HHIIBBHHH',40000,3130,1,1,offset,case['flags'],1024,0,0)+tcp_options+payload
pseudo=address+address+struct.pack('!BBH',0,6,len(tcp));check=checksum(pseudo+tcp)
tcp=tcp[:16]+struct.pack('!H',check)+tcp[18:]
ip=struct.pack('!BBHHHBBH4s4s',0x40+((20+len(ip_options))//4),0,20+len(ip_options)+len(tcp),1,0,64,6,0,address,address)+ip_options
with socket.socket(socket.AF_INET,socket.SOCK_RAW,socket.IPPROTO_RAW) as stream:
 stream.setsockopt(socket.IPPROTO_IP,socket.IP_HDRINCL,1)
 try:stream.sendto(ip+tcp,('127.0.0.1',3130))
 except OSError as error:
  # The intentionally unapproved port is rejected after counter evaluation.
  if error.errno not in (1,111):raise
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--image', required=True)
    parser.add_argument('--policy', type=Path, help='read-only local policy source under test')
    parser.add_argument('--header-probes', action='store_true', help='kernel-test zero-payload header boundaries')
    parser.add_argument('--watchdog-seconds', type=int, default=0, help='actual unchanged watchdog observation, minimum 660')
    args = parser.parse_args()
    if not re.fullmatch(r'sha256:[0-9a-f]{64}', args.image):
        parser.error('immutable image ID required')
    if args.policy and (args.policy.is_symlink() or not args.policy.is_file()):
        parser.error('policy must be a regular source file')
    if args.watchdog_seconds and not 660 <= args.watchdog_seconds <= 900:
        parser.error('watchdog observation must be between 660 and 900 seconds')
    prefix = 'task9-loopback-' + uuid.uuid4().hex[:16]
    containers = []
    result = {'kind': 'task9-loopback-teardown-regression', 'imageId': args.image,
              'network': 'none', 'providerCalls': 0,
              'policySourceSha256': hashlib.sha256(args.policy.read_bytes()).hexdigest() if args.policy else None}

    def pod(*argv, check=True):
        completed = subprocess.run(['podman', *argv], capture_output=True, text=True, timeout=30)
        if check and completed.returncode:
            raise RuntimeError('bounded_fixture_engine_operation_failed_' + argv[0] + '_' + str(completed.returncode))
        if len(completed.stdout) + len(completed.stderr) > 131072:
            raise RuntimeError('bounded_fixture_output_exceeded')
        return completed

    def execute(name, code, check=True):
        return pod('exec', '--user', '0:0', name, 'python3', '-c', code, check=check)

    try:
        owner = prefix + '-owner'; containers.append(owner)
        policy_mount = ['-v', str(args.policy.resolve()) + ':/fixture-policy.py:ro',
                        '-e', 'FIXTURE_POLICY=/fixture-policy.py'] if args.policy else []
        pod('run', '-d', '--name', owner, '--label', 'thoughtkhoral.task9=loopback-health',
            '--network', 'none', '--user', '0:0', '--cap-drop', 'ALL', '--cap-add', 'NET_ADMIN',
            '--read-only', '--security-opt', 'no-new-privileges', '--tmpfs', '/run',
            '--tmpfs', '/tmp', *policy_mount, '--entrypoint', 'python3', args.image, '-u', '-c', OWNER)
        for _ in range(50):
            if pod('exec', owner, 'test', '-f', '/run/fixture-ready', check=False).returncode == 0:
                break
            time.sleep(0.1)
        else:
            raise RuntimeError('original_policy_or_counter_installation_failed')
        proxy = prefix + '-proxy'; containers.append(proxy)
        pod('run', '-d', '--name', proxy, '--label', 'thoughtkhoral.task9=loopback-health',
            '--network', 'container:' + owner, '--pid', 'container:' + owner,
            '--user', '10004:10004', '--cap-drop', 'ALL', '--read-only',
            '--security-opt', 'no-new-privileges', '--tmpfs', '/tmp', args.image)
        listener = "import pathlib;assert any(x.split()[3]=='0A' and x.split()[1].endswith(':0C38') for x in pathlib.Path('/proc/net/tcp').read_text().splitlines()[1:])"
        for _ in range(50):
            if execute(owner, listener, check=False).returncode == 0:
                break
            time.sleep(0.1)
        else:
            raise RuntimeError('proxy_listener_unavailable')
        before = json.loads(execute(owner, SNAPSHOT).stdout)
        execute(owner, HEALTH)
        time.sleep(0.25)
        after = json.loads(execute(owner, SNAPSHOT).stdout)
        result.update(before=before, after=after,
                      teardownComplete=after['clientLastAck'] == 0)
        expected = {'socketless-established-reply', 'eligible-zero-payload',
                    'socketless-established-original', 'socketless-invalid',
                    'socketless-untracked', 'socketless-all', 'owned-established-reply', 'header-shape'}
        if set(after['counters']) != expected:
            raise RuntimeError('incomplete_packet_classification')
        result['approvedPredicateMatched'] = (after['counters']['eligible-zero-payload']
                                            - before['counters']['eligible-zero-payload']) > 0
        if not result['approvedPredicateMatched'] or any(
                after['counters'][name] != before['counters'][name]
                for name in ['socketless-established-original', 'socketless-invalid', 'socketless-untracked']):
            raise RuntimeError('approved_packet_classification_not_demonstrated')
        result['result'] = 'passed' if result['teardownComplete'] else 'failed'
        if result['teardownComplete'] and args.header_probes:
            sender = prefix + '-header-sender'; containers.append(sender)
            # This diagnostic sender has NET_RAW only in the owned network-none
            # namespace; no production worker/proxy gains a capability.
            pod('run', '-d', '--name', sender, '--label', 'thoughtkhoral.task9=loopback-health',
                '--network', 'container:' + owner, '--user', '0:0', '--cap-drop', 'ALL',
                '--cap-add', 'NET_RAW', '--read-only', '--security-opt', 'no-new-privileges',
                '--entrypoint', '/bin/sleep', args.image, '60')
            cases = [
                ('empty-timestamped-ack', 0, 12, 0, 16, 0, True),
                ('one-byte-payload', 0, 12, 1, 16, 0, False),
                ('same-length-data', 0, 0, 12, 16, 0, False),
                ('ip-options-and-data', 4, 4, 4, 16, 0, False),
                ('psh-ack', 0, 12, 0, 24, 0, False),
                ('longer-empty-header', 0, 16, 0, 16, 0, False),
                ('ip-options-empty', 4, 8, 0, 16, 0, False),
                ('reserved-header-bit', 0, 12, 0, 16, 1, False),
                ('shorter-empty-header', 0, 0, 0, 16, 0, False),
            ]
            header_results = []
            for name, ip_options, tcp_options, payload, flags, reserved, allowed in cases:
                initial = json.loads(execute(owner, SNAPSHOT).stdout)['counters']['header-shape']
                case = dict(ipOptions=ip_options, tcpOptions=tcp_options, payload=payload,
                            flags=flags, reserved=reserved)
                pod('exec', sender, 'python3', '-c', RAW_PACKET, json.dumps(case))
                actual = json.loads(execute(owner, SNAPSHOT).stdout)['counters']['header-shape'] - initial
                if actual != int(allowed):
                    raise RuntimeError('kernel_header_boundary_refusal_failed')
                header_results.append({'case': name, 'headerPredicateMatched': bool(actual)})
            result['kernelHeaderCases'] = header_results
        if result['teardownComplete'] and args.watchdog_seconds:
            guard = prefix + '-guard'; containers.append(guard)
            guard_code = "import os,runpy;runpy.run_path(os.environ.get('FIXTURE_POLICY','/opt/codex/codex-egress.py'),run_name='__main__')"
            pod('run', '-d', '--name', guard, '--label', 'thoughtkhoral.task9=loopback-health',
                '--network', 'none', '--user', '0:0', '--cap-drop', 'ALL', '--cap-add', 'NET_ADMIN',
                '--read-only', '--security-opt', 'no-new-privileges', '--tmpfs', '/run', '--tmpfs', '/tmp',
                *policy_mount, '--entrypoint', 'python3', args.image, '-u', '-c', guard_code)
            for _ in range(50):
                if pod('exec', guard, 'test', '-f', '/run/codex-policy-ready', check=False).returncode == 0:
                    break
                time.sleep(0.1)
            else:
                raise RuntimeError('actual_watchdog_policy_unavailable')
            guard_proxy = prefix + '-guard-proxy'; containers.append(guard_proxy)
            pod('run', '-d', '--name', guard_proxy, '--label', 'thoughtkhoral.task9=loopback-health',
                '--network', 'container:' + guard, '--pid', 'container:' + guard,
                '--user', '10004:10004', '--cap-drop', 'ALL', '--read-only',
                '--security-opt', 'no-new-privileges', '--tmpfs', '/tmp', args.image)
            dependent = prefix + '-dependent'; containers.append(dependent)
            pod('run', '-d', '--name', dependent, '--label', 'thoughtkhoral.task9=loopback-health',
                '--network', 'container:' + guard, '--pid', 'container:' + guard,
                '--user', '10003:10003', '--cap-drop', 'ALL', '--read-only',
                '--security-opt', 'no-new-privileges', '--entrypoint', '/bin/sleep', args.image, 'infinity')
            started = time.monotonic(); next_progress = 0
            while time.monotonic() - started < args.watchdog_seconds:
                for name in [guard, guard_proxy, dependent]:
                    state = json.loads(pod('inspect', '--format', '{{json .State}}', name).stdout)
                    if not state['Running'] or state['OOMKilled']:
                        raise RuntimeError('actual_watchdog_failed_during_observation')
                elapsed = time.monotonic() - started
                if elapsed >= next_progress:
                    print(json.dumps({'watchdogObservationSeconds': int(elapsed), 'running': True}), flush=True)
                    next_progress += 60
                time.sleep(min(5, max(0, args.watchdog_seconds - elapsed)))
            result['watchdogObservationSeconds'] = round(time.monotonic() - started, 3)
            pod('stop', '--time', '0', guard_proxy)
            for _ in range(50):
                states = [json.loads(pod('inspect', '--format', '{{json .State}}', name).stdout)
                          for name in [guard, dependent]]
                if all(not state['Running'] for state in states):
                    break
                time.sleep(0.2)
            else:
                raise RuntimeError('proxy_loss_did_not_stop_namespace_and_dependent')
            result['proxyLossStopsNamespaceAndDependent'] = True
    except (RuntimeError, subprocess.SubprocessError, ValueError) as error:
        result.update(result='fixture-error', reason=str(error) if isinstance(error, RuntimeError)
                      else 'bounded_fixture_execution_failed')
    finally:
        cleanup = True
        for name in reversed(containers):
            cleanup &= pod('rm', '--force', '--ignore', name, check=False).returncode == 0
        result['ownedCleanupPassed'] = cleanup
        if not cleanup:
            result['result'] = 'fixture-error'
    print(json.dumps(result, sort_keys=True))
    return 0 if result['result'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
