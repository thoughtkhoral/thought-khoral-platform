#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Host-only secret separation gate; never prints values or calls a provider."""
import os,pathlib,re,sys

def check(source_build=False):
    if not source_build and not re.fullmatch(r'(sha256:[0-9a-f]{64}|[^\s]+@sha256:[0-9a-f]{64})',os.environ.get('THOUGHT_KHORAL_CODEX_IMAGE','')):
        raise ValueError('an immutable worker image ID/digest is required')
    values=[]
    for suffix in ['PROVIDER_KEY_FILE','INVOCATION_KEY_FILE','CATALOG_BRIDGE_KEY_FILE']:
        name='THOUGHT_KHORAL_CODEX_'+suffix
        path=pathlib.Path(os.environ.get(name,''))
        if not path.is_absolute() or not path.is_file(): raise ValueError(name+' must identify a regular absolute file')
        with path.open('rb') as stream: raw=stream.read(4097)
        value=raw.rstrip(b'\r\n')
        if len(raw)>4096 or not re.fullmatch(rb'[\x21-\x7e]{16,4096}',value): raise ValueError(name+' has invalid credential format')
        if suffix=='CATALOG_BRIDGE_KEY_FILE' and not re.fullmatch(rb'[A-Za-z0-9._~+/=-]{32,4096}',value): raise ValueError('catalog bridge key has invalid format')
        values.append(value)
    if len(set(values))!=3: raise ValueError('provider, invocation and catalog bridge credentials must differ')

if __name__=='__main__':
    try:
        if sys.argv[1:] not in [[],['--source-build']]: raise ValueError('unexpected arguments')
        check(sys.argv[1:]==['--source-build'])
    except (OSError,ValueError) as error:
        raise SystemExit('Codex input validation failed: '+str(error))
    print('Codex local input separation verified; no provider call')
