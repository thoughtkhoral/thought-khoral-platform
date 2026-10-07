#!/usr/bin/env python3
"""Real local proxy tests; --kernel runs isolated Podman network fixtures too."""
import importlib.util
import pathlib
import socket
import ssl
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[1]
DETERMINISTIC = '--deterministic' in sys.argv
if DETERMINISTIC: sys.argv.remove('--deterministic')
KERNEL = '--kernel' in sys.argv
if KERNEL: sys.argv.remove('--kernel')

class ProxyPolicy(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path = ROOT/'proxy/codex-provider.py'
        if not path.exists(): return
        spec = importlib.util.spec_from_file_location('provider_proxy', path)
        cls.proxy = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.proxy)

    def test_closed_configuration_rejects_origin_override(self):
        self.assertTrue(hasattr(self,'proxy'), 'restricted proxy is not implemented')
        with tempfile.TemporaryDirectory() as tmp:
            p=pathlib.Path(tmp)/'config'
            p.write_text('{"connect":"evil.example:443","listen":"127.0.0.1:3128"}')
            result=subprocess.run([sys.executable,str(ROOT/'proxy/codex-provider.py'),'--check-config',str(p)],capture_output=True)
            self.assertNotEqual(result.returncode,0)

    def test_denied_connects_never_dial_upstream(self):
        self.assertTrue(hasattr(self,'proxy'), 'restricted proxy is not implemented')
        with self.proxy.Server(('127.0.0.1',0),self.proxy.Handler) as server:
            t=threading.Thread(target=server.serve_forever,daemon=True);t.start()
            try:
                with patch.object(self.proxy,'open_upstream',side_effect=AssertionError('denied target dialed')):
                    for target in ['evil.example:443','api.openai.com:80','127.0.0.1:443','[::1]:443','api.openai.com.evil:443','api.openai.com:443@evil','api.openai.com.:443']:
                        with self.subTest(target=target),socket.create_connection(server.server_address) as stream:
                            stream.sendall(f'CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n'.encode())
                            self.assertTrue(stream.recv(1024).startswith(b'HTTP/1.1 403 '))
            finally: server.shutdown();t.join()

    def test_resolved_nonpublic_addresses_never_dial(self):
        for address in ['127.0.0.1','10.0.0.1','::1','fe80::1','224.0.0.1','ff0e::1','240.0.0.1']:
            with self.subTest(address=address),patch.object(self.proxy.socket,'getaddrinfo',return_value=[(socket.AF_INET,socket.SOCK_STREAM,6,'',(address,443))]),patch.object(self.proxy.socket,'create_connection',side_effect=AssertionError('forbidden address dialed')):
                with self.assertRaises(OSError): self.proxy.open_upstream()

    def test_real_tls_verification_rejects_untrusted_upstream(self):
        self.assertTrue(hasattr(self,'proxy'), 'restricted proxy is not implemented')
        with tempfile.TemporaryDirectory() as tmp:
            p=pathlib.Path(tmp)
            subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-subj','/CN=api.openai.com','-addext','subjectAltName=DNS:api.openai.com','-keyout',str(p/'key'),'-out',str(p/'cert')],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            ctx=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);ctx.load_cert_chain(p/'cert',p/'key')
            listener=socket.socket();listener.bind(('127.0.0.1',0));listener.listen()
            def accept():
                conn,_=listener.accept()
                try:
                    with ctx.wrap_socket(conn,server_side=True): pass
                except ssl.SSLError: conn.close()
            t=threading.Thread(target=accept);t.start()
            def controlled(address,**kwargs):
                stream=socket.socket();stream.settimeout(3);stream.connect(listener.getsockname());return stream
            try:
                with patch.object(self.proxy.socket,'getaddrinfo',return_value=[(socket.AF_INET,socket.SOCK_STREAM,6,'',('93.184.216.34',443))]),patch.object(self.proxy.socket,'create_connection',side_effect=controlled):
                    with self.assertRaises(ssl.SSLError): self.proxy.open_upstream()
            finally: t.join(5);listener.close()

if __name__=='__main__':
    result=unittest.main(exit=False)
    if not result.result.wasSuccessful(): raise SystemExit(1)
    if KERNEL: subprocess.run(['bash',str(ROOT/'scripts/tests/codex-kernel.sh')],check=True)

    if DETERMINISTIC: subprocess.run([sys.executable,str(ROOT/'scripts/tests/deterministic-kernel.py')],check=True)
