import os
import pathlib
import subprocess
import tempfile
import unittest
ROOT=pathlib.Path(__file__).resolve().parents[2]
class Mediation(unittest.TestCase):
    def test_opt_in_adds_only_owner_scoped_worker_and_catalog_paths(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=pathlib.Path(tmp)
            for name in ['iptables','ip6tables']:
                (p/name).write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$RULES"\n')
            (p/'getent').write_text('''#!/bin/sh
case "$2" in
thought-khoral-room-gateway) echo '10.22.0.2 STREAM broker';;
thought-khoral-keycloak) echo '10.22.0.3 STREAM identity';;
thought-khoral-codex-agent) echo '10.23.0.4 STREAM worker';;
*) exit 1;;
esac
''')
            for f in p.iterdir(): f.chmod(0o755)
            result=subprocess.run(['sh', str(ROOT/'scripts/agent-egress.sh')], env=os.environ|{'PATH':str(p)+':'+os.environ['PATH'],'RULES':str(p/'rules'),'THOUGHT_KHORAL_CODEX_ENABLED':'true'},capture_output=True)
            self.assertEqual(result.returncode,0,result.stderr)
            rules=(p/'rules').read_text()
            self.assertIn('--uid-owner 10001 -p tcp -m conntrack --ctorigdst 10.23.0.4 --ctorigdstport 9091 --ctdir ORIGINAL -j ACCEPT',rules)
            self.assertIn('--uid-owner 10001 -p tcp -m conntrack --ctorigsrc 10.22.0.2 --ctorigdstport 9092 --ctdir REPLY -j ACCEPT',rules)
            self.assertNotIn('--uid-owner 10002 -p tcp',rules)
if __name__=='__main__': unittest.main()
