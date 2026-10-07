#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Closed CONNECT proxy. Never decrypts application traffic or logs requests."""
import ipaddress
import json
import pathlib
import select
import socket
import socketserver
import ssl
import sys
import threading
import time

POLICY = {'connect': 'api.openai.com:443', 'listen': '127.0.0.1:3128'}

def open_upstream():
    addresses = socket.getaddrinfo('api.openai.com', 443, type=socket.SOCK_STREAM)
    if not addresses:
        raise OSError('no provider address')
    # Resolve once. Refuse private, loopback, multicast and reserved destinations.
    address = addresses[0][4][0]
    resolved = ipaddress.ip_address(address)
    if not resolved.is_global or resolved.is_multicast or resolved.is_reserved:
        raise OSError('non-public provider address')
    # Validate the selected endpoint without transmitting a provider credential.
    # The worker independently validates TLS on the subsequently tunneled stream.
    context = ssl.create_default_context()
    with socket.create_connection((address, 443), timeout=5) as preflight:
        with context.wrap_socket(preflight, server_hostname='api.openai.com'):
            pass
    return socket.create_connection((address, 443), timeout=5)

class Server(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True
    slots = threading.BoundedSemaphore(16)

    def handle_error(self, request, client_address):
        pass  # Never print request data, credentials, hostnames or headers.

class Handler(socketserver.BaseRequestHandler):
    def respond(self, status):
        self.request.sendall(f'HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n'.encode())

    def handle(self):
        if not self.server.slots.acquire(blocking=False):
            self.respond('503 Busy'); return
        try:
            self.request.settimeout(5)
            data = b''
            while b'\r\n\r\n' not in data:
                block = self.request.recv(1024)
                if not block: return
                data += block
                if len(data) > 8192: self.respond('431 Too Large'); return
            headers, pending = data.split(b'\r\n\r\n', 1)
            first = headers.split(b'\r\n', 1)[0]
            if first == b'GET /health HTTP/1.1':
                self.respond('200 OK'); return
            if first not in [b'CONNECT api.openai.com:443 HTTP/1.1', b'CONNECT api.openai.com:443 HTTP/1.0']:
                self.respond('403 Forbidden'); return
            try:
                upstream = open_upstream()
            except (OSError, ValueError):
                self.respond('502 Unavailable'); return
            with upstream:
                self.request.sendall(b'HTTP/1.1 200 Connection Established\r\n\r\n')
                if pending: upstream.sendall(pending)
                deadline = time.monotonic() + 200
                remaining = 64 * 1024 * 1024
                while remaining > 0 and time.monotonic() < deadline:
                    readers, _, _ = select.select([upstream, self.request], [], [], 1)
                    for reader in readers:
                        payload = reader.recv(min(65536, remaining))
                        if not payload: return
                        remaining -= len(payload)
                        (self.request if reader is upstream else upstream).sendall(payload)
        except OSError:
            pass
        finally:
            self.server.slots.release()

def main():
    if sys.argv[1:] == ['--health']:
        with socket.create_connection(('127.0.0.1', 3128), timeout=1) as stream:
            stream.sendall(b'GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n')
            if not stream.recv(128).startswith(b'HTTP/1.1 200 '): raise SystemExit(1)
        return
    check = len(sys.argv) == 3 and sys.argv[1] == '--check-config'
    path = sys.argv[2] if check else '/opt/codex/codex-provider.conf'
    if json.loads(pathlib.Path(path).read_text()) != POLICY:
        raise SystemExit('invalid closed provider policy')
    if check: return
    with Server(('127.0.0.1', 3128), Handler) as server:
        server.serve_forever()

if __name__ == '__main__': main()
