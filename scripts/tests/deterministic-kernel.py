#!/usr/bin/env python3
"""Existing deterministic smoke through a strictly scoped Task8 fixture adapter."""
import os,pathlib,subprocess,tempfile,time
ROOT=pathlib.Path(__file__).resolve().parents[2]
PREFIX=f'task8-deterministic-{os.getpid()}'
PROXY='localhost/thought-khoral-codex-proxy:task8-fixture'
EGRESS='localhost/thought-khoral-agent-egress:dev'
containers=[];network=PREFIX+'-net'
def pod(*args,check=True):
 r=subprocess.run(['podman',*args],capture_output=True,text=True)
 if check and r.returncode:raise AssertionError(str(args[:3])+': '+r.stderr)
 return r

def run(suffix,*args):
 name=PREFIX+'-'+suffix;containers.append(name);pod('run','-d','--name',name,*args);return name

def wait(fn):
 for _ in range(40):
  if fn():return
  time.sleep(.2)
 raise AssertionError('fixture not ready')
try:
 pod('network','create','--internal','--ipv6','--subnet','10.248.83.0/24','--subnet','fd00:8:43::/64',network)
 with tempfile.TemporaryDirectory(prefix='task8-deterministic-') as tmp:
  tmp=pathlib.Path(tmp);tmp.chmod(0o755)
  server=tmp/'server.py';server.write_text('''import http.server,socket,threading,time
class Quiet(http.server.BaseHTTPRequestHandler):
 def do_GET(self):self.send_response(200);self.end_headers();self.wfile.write(b'task8')
 def log_message(self,*args):pass
class Six(http.server.HTTPServer):
 address_family=socket.AF_INET6
 def server_bind(self):self.socket.setsockopt(socket.IPPROTO_IPV6,socket.IPV6_V6ONLY,1);super().server_bind()
for cls,host,port in [(http.server.HTTPServer,'0.0.0.0',8080),(Six,'::',8080),(http.server.HTTPServer,'0.0.0.0',9091)]:
 srv=cls((host,port),Quiet);threading.Thread(target=srv.serve_forever,daemon=True).start()
while True:time.sleep(1)
''')
  target=run('target','--network',network,'--ip','10.248.83.20','--ip6','fd00:8:43::20','--entrypoint','python3','-v',str(tmp)+':/fixture:ro',PROXY,'/fixture/server.py')
  control=run('control','--network',network,'--entrypoint','sleep',EGRESS,'infinity')
  for address in ['10.248.83.20','[fd00:8:43::20]']:
   wait(lambda:pod('exec',control,'curl','--noproxy','*','--fail','--max-time','1','http://'+address+':8080/',check=False).returncode==0)
  def owner(suffix,enabled):
   args=['--network',network,'--cap-drop','ALL','--cap-add','NET_ADMIN','--read-only','--tmpfs','/tmp','--user','0:0','--add-host','thought-khoral-room-gateway:10.248.83.20','--add-host','thought-khoral-keycloak:10.248.83.20','--add-host','thought-khoral-codex-agent:10.248.83.20','-v',str(ROOT/'scripts/agent-egress.sh')+':/fixture-egress.sh:ro','--entrypoint','/bin/sh']
   if enabled:args+=['-e','THOUGHT_KHORAL_CODEX_ENABLED=true']
   name=run(suffix,*args,EGRESS,'/fixture-egress.sh','--hold')
   wait(lambda:pod('exec',name,'test','-f','/tmp/egress-ready',check=False).returncode==0)
   return name
  base=owner('base',False)
  shim=tmp/'podman-compose'
  shim.write_text('''#!/bin/sh
set -eu
[ "$1" = -f ]; shift 2
[ "$1" = exec ] && [ "$2" = -T ] && [ "$3" = --user ]; uid=$4;shift 4
[ "$1" = thought-khoral-agent-egress ];shift
exec podman exec --user "$uid" "$TASK8_OWNER" "$@"
''');shim.chmod(0o755)
  env=os.environ|{'PATH':str(tmp)+':'+os.environ['PATH'],'TASK8_OWNER':base}
  subprocess.run(['sh',str(ROOT/'scripts/smoke-agent-egress.sh')],env=env,check=True)
  for uid in ['10001','10002']:
   assert pod('exec','--user',uid+':'+uid,base,'curl','--noproxy','*','--max-time','2','http://10.248.83.20:9091/',check=False).returncode!=0
   assert pod('exec','--user',uid+':'+uid,base,'curl','--noproxy','*','--max-time','2','http://[fd00:8:43::20]:8080/',check=False).returncode!=0
  admitted=owner('admitted',True)
  pod('exec','--user','10001:10001',admitted,'curl','--noproxy','*','--fail','--max-time','2','http://10.248.83.20:9091/')
  assert pod('exec','--user','10002:10002',admitted,'curl','--noproxy','*','--max-time','2','http://10.248.83.20:9091/',check=False).returncode!=0
  # UID10001 listener: only replies to broker-initiated9092 traffic are admitted.
  mediator=run('mediator','--network','container:'+admitted,'--pid','container:'+admitted,'--user','10001:10001','--cap-drop','ALL','--read-only','--entrypoint','python3',PROXY,'-m','http.server','9092','--bind','0.0.0.0')
  ip=pod('inspect','--format','{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}',admitted).stdout.strip()
  wait(lambda:pod('exec',target,'python3','-c',f"import urllib.request;assert urllib.request.urlopen('http://{ip}:9092/',timeout=1).status==200",check=False).returncode==0)
  print('PASS isolated existing deterministic smoke; opt-in UID10001 worker9091 and broker-initiated9092 reply; UID10002 denied; controlled IPv6 denied',flush=True)
finally:
 for name in reversed(containers):pod('rm','-f',name,check=False)
 pod('network','rm',network,check=False)
