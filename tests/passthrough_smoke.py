"""Exercise real HTTP/UDP relay boundaries and the Lua game-thread facade."""
import http.client
import json
import os
import socket
import subprocess
import tempfile
import time
from pathlib import Path
from lupa import LuaRuntime, lua_type
ROOT=Path(__file__).resolve().parents[1]

def free_port(kind):
    with socket.socket(socket.AF_INET,kind) as s:
        s.bind(('127.0.0.1',0)); return s.getsockname()[1]

def network():
    config=dict(token='test-only-token-'*4,session='isolated-test',httpPort=free_port(socket.SOCK_STREAM),
                udpPort=free_port(socket.SOCK_DGRAM),metresPerStud=.3,hostOrigin=[0,0,0])
    with tempfile.TemporaryDirectory() as tmp:
        path=Path(tmp)/'session.json'; path.write_text(json.dumps(config))
        exe=ROOT/'bridge/target/debug'/('avatar-bridge.exe' if os.name=='nt' else 'avatar-bridge')
        with subprocess.Popen([str(exe),'--config',str(path)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True) as proc:
            try:
                assert 'ready' in proc.stdout.readline(), proc.stderr.read()
                with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as udp:
                    udp.connect(('127.0.0.1',config['udpPort'])); udp.settimeout(1)
                    def msg(kind,sequence,payload):
                        return dict(version=1,token=config['token'],session=config['session'],
                                    sequence=sequence,kind=kind,payload=payload)
                    def host(seq,enabled=True):
                        return msg('host',seq,dict(enabled=enabled,movement=[1,0],jump=False,colliders=[]))
                    def send(m):
                        udp.send(json.dumps(m).encode()); return json.loads(udp.recv(8192))
                    def post(m,headers=None):
                        c=http.client.HTTPConnection('127.0.0.1',config['httpPort'],timeout=2)
                        try:
                            c.request('POST','/exchange',json.dumps(m),headers or {'Content-Type':'application/json'})
                            r=c.getresponse(); return r.status,json.loads(r.read())
                        finally: c.close()
                    pose=dict(position=[1,2,3],velocity=[4,0,0],state='Running',parts=[dict(id='Head',
                        position=[1,2,4],size=[.6,.3,.3],right=[1,0,0],up=[0,0,1],back=[0,-1,0],color=[1,1,0])],
                        camera=dict(position=[1,-2,3],forward=[0,1,0],up=[0,0,1],fov=70))
                    assert send(host(1))['active'] is False
                    status,r=post(msg('guest',1,pose)); assert status==200 and r['active']
                    r=send(host(2)); assert r['payload']==pose and r['peerSequence']==1
                    assert not send(host(2))['ok']
                    wrong=host(3); wrong['token']='wrong'; assert not send(wrong)['ok']
                    wrong=host(3); wrong['version']=2; assert not send(wrong)['ok']
                    bad=json.loads(json.dumps(pose)); bad['parts'][0]['back']=[0,1,0]
                    assert post(msg('guest',2,bad))[0]==400
                    assert post(msg('guest',2,pose),{'Origin':'https://example.invalid'})[0]==400
                    assert post(host(3))[0]==400  # HTTP cannot publish host commands.
                    assert post(msg('guest',2,pose))[0]==200  # Rejections did not consume sequence.
                    assert send(host(3,False))['active'] is False
                    time.sleep(1.05)
                    r=send(host(4)); assert not r['active'] and r['payload'] is None
                    udp.send(b'x'*8193); assert not json.loads(udp.recv(8192))['ok']
                    # Oversized HTTP bodies are rejected before allocation/read.
                    c=http.client.HTTPConnection('127.0.0.1',config['httpPort'],timeout=2)
                    c.request('POST','/exchange',body='',headers={'Content-Length':'999999999'})
                    assert c.getresponse().status==400; c.close()
                    # A silent request must release the single HTTP worker.
                    with socket.create_connection(('127.0.0.1',config['httpPort']),timeout=2) as slow:
                        slow.sendall(b'POST /exchange HTTP/1.1\r\n')
                        assert slow.recv(1)==b''
                    assert post(msg('guest',3,pose))[0]==200
            finally:
                proc.terminate(); proc.wait(timeout=3)
    print('Real Rust HTTP/UDP exchange, validation, stale peers and bounded request deadline passed')

def facade():
    lua=LuaRuntime(unpack_returned_tuples=True)
    def native(v):
        if lua_type(v)=='table':
            keys=list(v.keys())
            if keys and all(isinstance(k,int) for k in keys): return [native(v[i]) for i in range(1,len(keys)+1)]
            return {k:native(x) for k,x in v.items()}
        return v
    g=lua.globals(); sent=[]
    g.pyEncode=lambda v:json.dumps(native(v))
    g.pyDecode=lambda s:lua.table_from(json.loads(s),recursive=True)
    g.savePacket=lambda s:sent.append(json.loads(s))
    lua.execute('''
queue={}; closed=false
jsonEncode=function(v) return pyEncode(v) end
jsonDecode=function(s) return pyDecode(s) end
require=function(name)
 assert(name=='socket')
 return {udp=function() return {
 settimeout=function(_,v) assert(v==0) end,
 setpeername=function(_,host,port) assert(host=='127.0.0.1' and port==28742); return true end,
 send=function(_,s) savePacket(s); return #s end,
 receive=function() if #queue==0 then return nil,'timeout' end return table.remove(queue,1) end,
 close=function() closed=true end} end}
end
''')
    g.bridge=lua.execute((ROOT/'mod/lua/ge/extensions/avatarSandbox/passthrough.lua').read_text())
    lua.execute('''
assert(bridge.start({token=string.rep('x',48),session='lua-test',udpPort=28742}))
assert(bridge.setEnabled(true)); assert(bridge.setInput(1,1,true))
assert(bridge.setColliders({{id=1,position={0,0,-0.15},size={60,60,0.3}}}))
bridge.onUpdate(0.05,0.05)
queue[1]=jsonEncode({ok=true,version=1,session='lua-test',sequence=1,active=true,payload={position={1,2,3}}})
bridge.onUpdate(0.01,0.01); assert(bridge.getPose().position[1]==1)
bridge.onUpdate(0.05,0); assert(bridge.getPose()==nil)
bridge.onUpdate(1.1,0.01); assert(not bridge.status().active)
assert(bridge.setInput(0,1,false)); bridge.onUpdate(0.05,0.05)
bridge.onClientEndMission(); assert(closed and not bridge.status().connected)
''')
    assert abs(sum(x*x for x in sent[0]['payload']['movement'])-1)<1e-12
    assert sent[0]['payload']['jump']
    assert any(not m['payload']['enabled'] and m['payload']['movement']==[0,0] for m in sent)
    assert not sent[-1]['payload']['enabled']
    print('Lua nonblocking facade, input normalization, pause, stale pose and mission cleanup passed')
if __name__=='__main__': network(); facade()
