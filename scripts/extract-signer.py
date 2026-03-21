#!/usr/bin/env python3
"""Extract the EVM signer address from a repobox-signed git commit.
Usage: git cat-file commit <hash> | python3 extract-signer.py
Outputs the recovered address (checksummed) or nothing if unsigned/invalid.
"""
import sys
import re

def main():
    commit_text = sys.stdin.read()
    
    # Extract gpgsig hex (130 hex chars = 65 bytes = r(32) + s(32) + v(1))
    sig_match = re.search(r'[0-9a-fA-F]{130}', 
                          '\n'.join(re.findall(r'(?:gpgsig |^ )(.+)', commit_text, re.MULTILINE)))
    if not sig_match:
        return
    
    sig_hex = sig_match.group(0)
    sig_bytes = bytes.fromhex(sig_hex)
    if len(sig_bytes) != 65:
        return
    
    # Strip gpgsig header from commit to get signed data
    lines = commit_text.split('\n')
    result = []
    skip = False
    for line in lines:
        if line.startswith('gpgsig '):
            skip = True
            continue
        if skip and line.startswith(' '):
            continue
        if skip:
            skip = False
        result.append(line)
    
    signed_data = '\n'.join(result)
    
    # Keccak256 hash of the signed data
    from eth_hash.auto import keccak
    digest = keccak(signed_data.encode('utf-8'))
    
    # ecrecover
    r = int.from_bytes(sig_bytes[0:32], 'big')
    s = int.from_bytes(sig_bytes[32:64], 'big')
    v = sig_bytes[64]
    if v < 27:
        v += 27
    
    from eth_account import Account
    try:
        addr = Account._recover_hash(digest, vrs=(v, r, s))
        print(addr)
    except Exception:
        pass

if __name__ == '__main__':
    main()
