"""Synthetic UDP host for a real local Studio guest; not a BeamNG emulator."""
import argparse
import json
import socket
import time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--config',type=Path,default=ROOT/'imports/passthrough-session.json')
    p.add_argument('--duration',type=float,default=8)
    p.add_argument('--move',type=float,nargs=2,default=[0,0])
    p.add_argument('--jump',action='store_true')
    p.add_argument('--crash',action='store_true',help='Omit graceful disable to test lease expiry')
    p.add_argument('--output',type=Path,default=ROOT/'exports/passthrough-observation.json')
    args=p.parse_args(); config=json.loads(args.config.read_text())
    origin=config['hostOrigin']
    boxes=[dict(id=1,position=[origin[0],origin[1],origin[2]-0.15],size=[60,60,0.3]),
           dict(id=2,position=[origin[0]+8,origin[1],origin[2]+2],size=[0.3,12,4])]
    start=time.monotonic(); observations=[]; sequence=int(time.time()*1000)
    with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as s:
        s.connect(('127.0.0.1',config['udpPort'])); s.settimeout(.1)
        while time.monotonic()-start < args.duration:
            sequence+=1; now=time.monotonic()-start
            message=dict(version=1,token=config['token'],session=config['session'],sequence=sequence,
                         kind='host',payload=dict(enabled=True,movement=args.move,jump=args.jump,colliders=boxes))
            s.send(json.dumps(message).encode())
            try:
                reply=json.loads(s.recv(8192))
                if not reply.get('ok'): raise RuntimeError(reply.get('error'))
                if reply.get('payload'):
                    observations.append(dict(t=now,active=reply['active'],
                        guestSequence=reply['peerSequence'],payload=reply['payload']))
            except socket.timeout: pass
            time.sleep(.05)
        message['sequence']=sequence+1; message['payload']['enabled']=False
        message['payload']['movement']=[0,0]; message['payload']['jump']=False
        if not args.crash: s.send(json.dumps(message).encode())
    args.output.parent.mkdir(exist_ok=True)
    args.output.write_text(json.dumps(observations,indent=2))
    unique=len({r['guestSequence'] for r in observations})
    print(f'Received {len(observations)} observations, {unique} distinct native snapshots; {args.output}')
    if unique < 2: raise SystemExit('No advancing Studio guest verified')
if __name__=='__main__': main()
