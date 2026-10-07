#!/usr/bin/env python3
import os,pathlib,subprocess,tempfile,unittest,json
ROOT=pathlib.Path(__file__).resolve().parents[2]
class Inputs(unittest.TestCase):
 def test_three_secrets_must_be_distinct_before_compose(self):
  with tempfile.TemporaryDirectory() as tmp:
   p=pathlib.Path(tmp);files=[]
   for name,value in [('provider','synthetic-task8-provider-key-123456'),('invocation','synthetic-task8-invocation-key-4567'),('bridge','synthetic-task8-bridge-secret-7890123')]:
    f=p/name;f.write_text(value);files.append(str(f))
   env=os.environ|dict(zip(['THOUGHT_KHORAL_CODEX_PROVIDER_KEY_FILE','THOUGHT_KHORAL_CODEX_INVOCATION_KEY_FILE','THOUGHT_KHORAL_CODEX_CATALOG_BRIDGE_KEY_FILE'],files))
   env|={'THOUGHT_KHORAL_CODEX_IMAGE':'sha256:'+'a'*64}
   def check():return subprocess.run(['python3',str(ROOT/'scripts/check-codex-inputs.py')],env=env,capture_output=True)
   self.assertEqual(check().returncode,0,'valid host inputs must pass')
   (p/'bridge').write_text((p/'provider').read_text())
   self.assertNotEqual(check().returncode,0,'provider credential would reach broker')
   (p/'bridge').write_text('synthetic-task8-bridge-secret-7890123')
   env['THOUGHT_KHORAL_CODEX_IMAGE']='localhost/worker:mutable'
   self.assertNotEqual(check().returncode,0)
if __name__=='__main__':unittest.main()
