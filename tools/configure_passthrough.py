"""Create private session config and a Studio Command Bar installer; never publish either."""
import argparse
import json
import math
import os
import secrets
import uuid
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]

def lua_string(text):
    # Lua long strings preserve source verbatim; choose an absent delimiter.
    equals = ''
    while ']' + equals + ']' in text:
        equals += '='
    return '[' + equals + '[' + text + ']' + equals + ']'

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rotate',action='store_true',help='Replace private session; restart both peers')
    parser.add_argument('--http-port', type=int, default=28741)
    parser.add_argument('--udp-port', type=int, default=28742)
    parser.add_argument('--host-origin', nargs=3, type=float, default=[0,0,0])
    args = parser.parse_args()
    if any(not 1 <= n <= 65535 for n in (args.http_port,args.udp_port)):
        parser.error('Ports must be between 1 and 65535')
    if any(not math.isfinite(x) or abs(x)>1e6 for x in args.host_origin):
        parser.error('Host origin must be finite and within one million metres')
    config = dict(token=secrets.token_hex(24), session=uuid.uuid4().hex,
                  httpPort=args.http_port, udpPort=args.udp_port,
                  metresPerStud=0.3, hostOrigin=args.host_origin)
    destination = ROOT/'imports/passthrough-session.json'
    destination.parent.mkdir(exist_ok=True)
    # Refuse accidental rotation of an active session; remove old file explicitly.
    with destination.open('w' if args.rotate else 'x') as f:
        os.chmod(destination, 0o600)
        json.dump(config, f, indent=2, allow_nan=False)
    source = ROOT/'tools/roblox'
    installer = '\n'.join([
        'local SESSION_CONFIG = '+lua_string(json.dumps(config)),
        'local SERVER_SOURCE = '+lua_string((source/'passthrough.server.lua').read_text()),
        'local CLIENT_SOURCE = '+lua_string((source/'passthrough.client.lua').read_text()),
        (source/'setup_passthrough.lua').read_text()])
    output = ROOT/'exports/install_passthrough.lua'
    output.parent.mkdir(exist_ok=True)
    output.write_text(installer); output.chmod(0o600)
    print(f'Private config: {destination}\nPrivate Studio installer: {output}')
if __name__ == '__main__':
    main()
