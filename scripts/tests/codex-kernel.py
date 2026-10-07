#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Isolated synthetic kernel/proxy proof. Never starts the actual worker service."""
import json
import os
import pathlib
import shutil
import subprocess
import tempfile
import time

ROOT=pathlib.Path(__file__).resolve().parents[2]
PREFIX=f'task8-codex-{os.getpid()}'
IMAGE='localhost/thought-khoral-codex-proxy:task8-fixture'
containers=[]; networks=[]; volumes=[]

def cmd(*args,check=True):
    r=subprocess.run(args,capture_output=True,text=True)
    if check and r.returncode:
        raise AssertionError(f'{args[:4]} failed ({r.returncode}): {r.stderr[-2000:]} {r.stdout[-1000:]}')
    return r

def pod(*args,**kw): return cmd('podman',*args,**kw)
def run(name,*args):
    name=PREFIX+'-'+name
    containers.append(name)
    return pod('run','-d','--name',name,*args).stdout.strip(),name

def exec_(name,uid,code,check=True):
    return pod('exec','--user',f'{uid}:{uid}',name,'python3','-c',code,check=check)

def wait_for(fn,description):
    for _ in range(50):
        if fn(): return
        time.sleep(.2)
    raise AssertionError(description)

def stopped(name): return pod('inspect','--format','{{.State.Running}}',name).stdout.strip()=='false'

def connect_code(address,port):
    return f"import socket; s=socket.create_connection(({address!r},{port}),timeout=1);s.close()"

with tempfile.TemporaryDirectory(prefix='task8-codex-') as tmp:
    tmp=pathlib.Path(tmp)
    tmp.chmod(0o755)
    try:
        context=tmp/'context'/'thought-khoral-platform'
        for directory in ['proxy','scripts']:
            (context/directory).mkdir(parents=True)
        for relative in ['proxy/codex-provider.py','proxy/codex-provider.conf','proxy/THIRD_PARTY_NOTICES.md','proxy/debian-packages-aarch64.lock','scripts/codex-egress.py','scripts/codex-egress.sh']:
            shutil.copy2(ROOT/relative,context/relative)
        pod('build','--quiet','-t',IMAGE,'-f',str(ROOT/'containers/codex-provider-proxy.Containerfile'),str(context.parent))
        # Deliberately isolated documentation-purpose provider IP. No public route.
        net=PREFIX+'-net'; networks.append(net)
        pod('network','create','--internal','--ipv6','--subnet','93.184.216.0/24','--subnet','fd00:8:42::/64',net)
        cmd('openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-subj','/CN=api.openai.com','-addext','subjectAltName=DNS:api.openai.com','-keyout',str(tmp/'key.pem'),'-out',str(tmp/'cert.pem'))
        (tmp/'key.pem').chmod(0o600)
        (tmp/'cert.pem').chmod(0o644)
        server=tmp/'target.py'
        server.write_text('''import http.server,socket,ssl,threading,time
received=[]
class Quiet(http.server.BaseHTTPRequestHandler):
 def do_GET(self):
  self.send_response(200);self.end_headers();self.wfile.write(str(len(received)).encode() if self.path=="/dns-count" else b"task8-controlled-target")
 def log_message(self,*args): pass
class Six(http.server.HTTPServer):
 address_family=socket.AF_INET6
 def server_bind(self):
  self.socket.setsockopt(socket.IPPROTO_IPV6,socket.IPV6_V6ONLY,1);super().server_bind()
for cls,host,port,tls in [(http.server.HTTPServer,"0.0.0.0",80,False),(Six,"::",80,False),(http.server.HTTPServer,"0.0.0.0",443,True)]:
 srv=cls((host,port),Quiet)
 if tls:
  ctx=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);ctx.load_cert_chain('/fixture/cert.pem','/fixture/key.pem');srv.socket=ctx.wrap_socket(srv.socket,server_side=True)
 threading.Thread(target=srv.serve_forever,daemon=True).start()
def dns_udp():
 s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM);s.bind(('0.0.0.0',53))
 while True:
  data,addr=s.recvfrom(512);received.append(data);s.sendto(b'task8-dns',addr)
def dns_tcp():
 s=socket.socket();s.bind(('0.0.0.0',53));s.listen()
 while True:
  conn,addr=s.accept()
  with conn: received.append(conn.recv(512));conn.sendall(b'task8-dns')
threading.Thread(target=dns_udp,daemon=True).start()
threading.Thread(target=dns_tcp,daemon=True).start()
while True: time.sleep(1)
''')
        _,target=run('target','--network',net,'--ip','93.184.216.34','--ip6','fd00:8:42::34','--user','0:0','-v',f'{tmp}:/fixture:ro','--entrypoint','python3',IMAGE,'/fixture/target.py')
        _,control=run('positive-control','--network',net,'--entrypoint','sleep',IMAGE,'infinity')
        for address in ['93.184.216.34','fd00:8:42::34']:
            wait_for(lambda:exec_(control,10003,connect_code(address,80),check=False).returncode==0,'controlled target not reachable before kernel policy')
        dns_codes = ["import socket;s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM);s.settimeout(1);s.connect(('93.184.216.34',53));s.send(b'task8');assert s.recv(16)==b'task8-dns'", "import socket;s=socket.create_connection(('93.184.216.34',53),timeout=1);s.sendall(b'task8');assert s.recv(16)==b'task8-dns'"]
        for code in dns_codes: exec_(control,10003,code)
        _,owner=run('owner','--network',net,'--ip','93.184.216.10','--ip6','fd00:8:42::10','--add-host','api.openai.com:93.184.216.34','--add-host','thought-khoral-codex-provider-proxy:127.0.0.1','--cap-drop','ALL','--cap-add','NET_ADMIN','--user','0:0','--read-only','--tmpfs','/run','--tmpfs','/tmp','--entrypoint','/bin/sh',IMAGE,'/opt/codex/codex-egress.sh')
        wait_for(lambda:pod('exec',owner,'test','-f','/run/codex-policy-ready',check=False).returncode==0,'policy did not install')
        _,proxy=run('proxy','--network',f'container:{owner}','--pid',f'container:{owner}','--user','10004:10004','--cap-drop','ALL','--read-only','--tmpfs','/tmp','-e','SSL_CERT_FILE=/fixture/cert.pem','-v',f'{tmp}:/fixture:ro',IMAGE)
        wait_for(lambda:pod('exec',proxy,'python3','/opt/codex/codex-provider.py','--health',check=False).returncode==0,'proxy did not start')
        # Exercise the real packaging wrapper; only the worker transport/verifier
        # boundary is synthetic. Actual package verification has a separate test.
        readiness_context=tmp/'readiness';readiness_context.mkdir()
        shutil.copy2(ROOT/'scripts/codex-start.sh',readiness_context/'codex-start.sh')
        (readiness_context/'worker.py').write_text("""#!/usr/bin/python3
import http.server,sys
if '--verify-package' in sys.argv:
 print('worker package verified; codex-cli 0.160.0; uid/gid 10003; no inference')
 print('tool policy verified; exposed tools: []')
 raise SystemExit(0)
class Handler(http.server.BaseHTTPRequestHandler):
 protocol_version='HTTP/1.1'
 def do_GET(self):
  assert self.headers['Authorization']=='Bearer synthetic-task8-invocation-5678'
  self.send_response(200);self.end_headers()
 def log_message(self,*args):pass
http.server.HTTPServer(('0.0.0.0',9091),Handler).serve_forever()
""")
        (readiness_context/'Containerfile').write_text(f"""FROM {IMAGE}
USER 0:0
RUN mkdir -p /opt/platform /opt/thought-khoral-codex/workspace /var/lib/thought-khoral-codex/native /var/lib/thought-khoral-codex/receipts /run/codex-admission /run/secrets && chown 10003:10003 /var/lib/thought-khoral-codex/native /var/lib/thought-khoral-codex/receipts /run/codex-admission && chmod 700 /var/lib/thought-khoral-codex/native /var/lib/thought-khoral-codex/receipts && chmod 755 /run/codex-admission && chmod 555 /opt/thought-khoral-codex/workspace && printf synthetic-task8-provider-1234 > /run/secrets/codex-provider && printf synthetic-task8-invocation-5678 > /run/secrets/codex-invocation && chmod 444 /run/secrets/*
COPY codex-start.sh /opt/platform/codex-start.sh
COPY --chmod=0555 worker.py /usr/local/bin/thought-khoral-codex-agent
USER 10003:10003
ENTRYPOINT ["/bin/bash","/opt/platform/codex-start.sh"]
""")
        readiness_image='localhost/'+PREFIX+'-readiness'
        pod('build','--quiet','-t',readiness_image,str(readiness_context))
        immutable=pod('image','inspect','--format','{{.Id}}',readiness_image).stdout.strip()
        if not immutable.startswith('sha256:'):immutable='sha256:'+immutable
        admission_volume=PREFIX+'-admission';volumes.append(admission_volume);pod('volume','create',admission_volume)
        state_mounts=[]
        for part in ['native','receipts']:
            name=PREFIX+'-'+part;volumes.append(name);pod('volume','create',name)
            state_mounts+=['-v',name+':/var/lib/thought-khoral-codex/'+part]
        secret_mounts=[]
        for part,value in [('provider','synthetic-task8-provider-1234'),('invocation','synthetic-task8-invocation-5678')]:
            file=tmp/('codex-'+part);file.write_text(value);file.chmod(0o444)
            secret_mounts+=['-v',str(file)+':/run/secrets/codex-'+part+':ro']
        _,worker=run('worker-synthetic','--network',f'container:{owner}','--pid',f'container:{owner}','--user','10003:10003','--cap-drop','ALL','--read-only','--tmpfs','/tmp',*state_mounts,*secret_mounts,'-v',admission_volume+':/run/codex-admission','-e','THOUGHT_KHORAL_CODEX_IMAGE='+immutable,'-e','THOUGHT_KHORAL_CODEX_ISOLATION_VERIFIED=1',readiness_image)
        try:
            wait_for(lambda:pod('exec',worker,'test','-f','/run/codex-admission/admission.json',check=False).returncode==0,'real wrapper did not publish readiness')
        except AssertionError:
            print(pod('logs',worker,check=False).stderr,flush=True)
            print(pod('logs',owner,check=False).stderr,flush=True)
            raise
        def admission():
            return pod('run','--rm','--name',PREFIX+'-readiness-read','--network','none','--user','10004:10004','--cap-drop','ALL','--read-only','-v',admission_volume+':/read:ro','--entrypoint','cat',IMAGE,'/read/admission.json').stdout
        def assert_admission(enabled):
            script="import{readConversationAdmission as read}from "+json.dumps((ROOT/'ui/codex-admission.js').as_uri())+";import fs from 'node:fs';const text=fs.readFileSync(0,'utf8');const value=await read(async()=>({ok:true,text:async()=>text}));if(Boolean(value)!=="+str(enabled).lower()+")process.exit(1);"
            r=subprocess.run(['node','--input-type=module','-e',script],input=admission(),text=True,capture_output=True)
            assert r.returncode==0,r.stderr
        assert_admission(True)
        exec_(worker,10003,"import socket;assert socket.gethostbyname('thought-khoral-codex-provider-proxy')=='127.0.0.1'")
        for address in ['93.184.216.34','fd00:8:42::34']:
            assert exec_(worker,10003,connect_code(address,80),check=False).returncode!=0, 'worker bypassed kernel egress'
        assert exec_(worker,10003,connect_code('93.184.216.34',443),check=False).returncode!=0, 'worker connected directly to provider'
        for code in dns_codes: assert exec_(worker,10003,code,check=False).returncode!=0
        exec_(control,10003,"import urllib.request;assert urllib.request.urlopen('http://93.184.216.34/dns-count').read()==b'2'")
        code='''import socket,ssl
s=socket.create_connection(('127.0.0.1',3128),timeout=4)
s.sendall(b'CONNECT api.openai.com:443 HTTP/1.1\\r\\nHost: api.openai.com:443\\r\\n\\r\\n')
assert s.recv(4096).startswith(b'HTTP/1.1 200 ')
ctx=ssl.create_default_context(cafile='/fixture/cert.pem')
with ctx.wrap_socket(s,server_hostname='api.openai.com') as tls:
 tls.sendall(b'GET /task8 HTTP/1.1\\r\\nHost: api.openai.com\\r\\nConnection: close\\r\\n\\r\\n')
 data=b''
 while True:
  part=tls.recv(4096)
  if not part:break
  data+=part
 assert b'task8-controlled-target' in data
'''
        exec_(proxy,10003,"open('/fixture/cert.pem').read()")
        # The synthetic worker uses the real proxy and normal end-to-end TLS.
        # Execute in proxy filesystem as worker UID; same network policy and no caps.
        exec_(proxy,10003,code)
        for authority in ['evil.example:443','api.openai.com:80','93.184.216.34:443','[fd00:8:42::34]:443']:
            exec_(worker,10003,f"import socket;s=socket.create_connection(('127.0.0.1',3128));s.sendall(b'CONNECT {authority} HTTP/1.1\\r\\nHost: x\\r\\n\\r\\n');assert s.recv(1024).startswith(b'HTTP/1.1 403 ')")
        print('PASS real kernel IPv4/IPv6/direct-provider denial; exact CONNECT with controlled verified TLS',flush=True)
        pod('stop','--time','0',proxy)
        wait_for(lambda:stopped(owner),'owner survived proxy loss')
        wait_for(lambda:stopped(worker),'worker survived owner/proxy loss')
        time.sleep(5.1)
        assert_admission(False)
        print('PASS proxy loss stops real owner and real wrapper; persisted readiness expires and cold bootstrap rejects it',flush=True)
        # Fresh namespace: removing an OUTPUT jump is detected and stops dependents.
        _,owner2=run('owner-policy','--network',net,'--cap-drop','ALL','--cap-add','NET_ADMIN','--user','0:0','--read-only','--tmpfs','/run','--tmpfs','/tmp','--entrypoint','/bin/sh',IMAGE,'/opt/codex/codex-egress.sh')
        wait_for(lambda:pod('exec',owner2,'test','-f','/run/codex-policy-ready',check=False).returncode==0,'second policy missing')
        _,proxy2=run('proxy-policy','--network',f'container:{owner2}','--pid',f'container:{owner2}','--user','10004:10004','--cap-drop','ALL','--read-only',IMAGE)
        _,worker2=run('worker-policy','--network',f'container:{owner2}','--pid',f'container:{owner2}','--user','10003:10003','--cap-drop','ALL','--read-only','--entrypoint','sleep',IMAGE,'infinity')
        pod('exec',owner2,'ip6tables','-w','-D','OUTPUT','-j','THOUGHT_CODEX_EGRESS')
        wait_for(lambda:stopped(owner2),'owner survived IPv6 policy loss')
        wait_for(lambda:stopped(worker2),'worker survived policy loss')
        print('PASS policy loss stops real owner and synthetic worker PID namespace',flush=True)
    finally:
        for name in reversed(containers): pod('rm','-f',name,check=False)
        for name in reversed(networks): pod('network','rm',name,check=False)
        for name in reversed(volumes): pod('volume','rm',name,check=False)
