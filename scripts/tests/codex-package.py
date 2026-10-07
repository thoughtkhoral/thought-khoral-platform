#!/usr/bin/env python3
"""Real worker package/startup refusal tests using only Task8 named volumes."""
import argparse,os,pathlib,subprocess,tempfile
ROOT=pathlib.Path(__file__).resolve().parents[2]
PREFIX=f'task8-package-{os.getpid()}'
parser=argparse.ArgumentParser()
parser.add_argument('--image',default=os.environ.get('TASK8_WORKER_IMAGE','localhost/thought-khoral-codex-agent:task6-final'))
parser.add_argument('--expect-verified',action='store_true',default=os.environ.get('TASK8_EXPECT_VERIFIED')=='1')
args=parser.parse_args()
IMAGE=args.image
volumes=[]

def pod(*args,check=True):
 r=subprocess.run(['podman',*args],text=True,capture_output=True)
 if check and r.returncode: raise AssertionError(r.stderr+r.stdout)
 return r
IMMUTABLE=pod('image','inspect','--format','{{.Id}}',IMAGE).stdout.strip()
if not IMMUTABLE.startswith('sha256:'):IMMUTABLE='sha256:'+IMMUTABLE
try:
 for part in ['native','receipts','admission','secrets']:
  v=PREFIX+'-'+part;pod('volume','create',v);volumes.append(v)
 mounts=[]
 for part,dest in [('native','/var/lib/thought-khoral-codex/native'),('receipts','/var/lib/thought-khoral-codex/receipts'),('admission','/run/codex-admission'),('secrets','/run/secrets')]: mounts+=['-v',PREFIX+'-'+part+':'+dest+':nocopy']
 def setup(script):pod('run','--rm','--name',PREFIX+'-prepare','--network','none','--user','0:0',*mounts,'--entrypoint','bash',IMAGE,'-c',script)
 setup('chown 10003:10003 /var/lib/thought-khoral-codex/native /var/lib/thought-khoral-codex/receipts /run/codex-admission; chmod 700 /var/lib/thought-khoral-codex/native /var/lib/thought-khoral-codex/receipts; chmod 755 /run/codex-admission; printf synthetic-task8-provider-1234 > /run/secrets/codex-provider; printf synthetic-task8-invocation-5678 > /run/secrets/codex-invocation; chmod 444 /run/secrets/*; touch /var/lib/thought-khoral-codex/native/task8-sentinel /var/lib/thought-khoral-codex/receipts/task8-sentinel /run/codex-admission/task8-sentinel')
 def check(extra=(),reference=IMMUTABLE):return pod('run','--rm','--name',PREFIX+'-check','--network','none','--user','10003:10003','--cap-drop','ALL','--read-only','--tmpfs','/tmp',*mounts,'-v',str(ROOT/'scripts/codex-start.sh')+':/start.sh:ro','-e','THOUGHT_KHORAL_CODEX_ISOLATION_VERIFIED=1','-e','THOUGHT_KHORAL_CODEX_IMAGE='+reference,*extra,'--entrypoint','bash',IMAGE,'/start.sh','--check',check=False)
 for mutate,restore,case in [
 ('mv /run/secrets/codex-provider /run/secrets/held','mv /run/secrets/held /run/secrets/codex-provider','missing provider key'),
 ('touch /var/lib/thought-khoral-codex/native/config.toml','rm /var/lib/thought-khoral-codex/native/config.toml','imported native config'),
 ('touch /var/lib/thought-khoral-codex/native/auth.json','rm /var/lib/thought-khoral-codex/native/auth.json','imported native auth'),
 ('chmod 755 /var/lib/thought-khoral-codex/native','chmod 700 /var/lib/thought-khoral-codex/native','wrong native mode'),
 ('chown 0:0 /var/lib/thought-khoral-codex/receipts','chown 10003:10003 /var/lib/thought-khoral-codex/receipts','wrong receipts owner'),
 ('cp /run/secrets/codex-provider /run/secrets/codex-invocation','printf synthetic-task8-invocation-5678 > /run/secrets/codex-invocation','shared credentials')]:
  setup(mutate);r=check();assert r.returncode!=0,case;setup(restore);print('PASS startup rejects '+case,flush=True)
 assert check(reference='localhost/worker:mutable').returncode!=0
 print('PASS startup rejects mutable image tag',flush=True)
 with tempfile.TemporaryDirectory(prefix='task8-cli-version-') as tmp:
  pathlib.Path(tmp).chmod(0o755)
  wrong=pathlib.Path(tmp)/'codex';wrong.write_text('#!/bin/sh\nprintf "codex-cli 0.159.0\\n"\n');wrong.chmod(0o755)
  assert check(('-v',str(wrong)+':/usr/local/bin/codex:ro')).returncode!=0
 print('PASS startup rejects wrong CLI version independently of tool evidence',flush=True)
 setup('printf stale > /run/codex-admission/admission.json; chown 10003:10003 /run/codex-admission/admission.json; chmod 755 /var/lib/thought-khoral-codex/native')
 assert check().returncode!=0
 pod('run','--rm','--name',PREFIX+'-stale','--network','none','--user','10003:10003',*mounts,'--entrypoint','test',IMAGE,'!','-e','/run/codex-admission/admission.json')
 setup('chmod 700 /var/lib/thought-khoral-codex/native')
 print('PASS failed restart removes stale admission before verifier',flush=True)
 result=check()
 if args.expect_verified:
  assert result.returncode==0,result.stderr;assert 'packaging verified' in result.stdout
  print('PASS corrected image package and startup readiness; no inference',flush=True)
 else:
  assert result.returncode!=0,'version-only package was admitted'
  print('PASS version-only image fails closed without verified empty tool policy',flush=True)
finally:
 for name in [PREFIX+'-prepare',PREFIX+'-check',PREFIX+'-stale']:pod('rm','-f',name,check=False)
 for name in volumes:pod('volume','rm',name,check=False)
