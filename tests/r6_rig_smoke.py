"""Run against the downloaded source GLB; validate missing-joint rejection too."""
import importlib.util
import json
import struct
import sys
import tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('rig',root/'tools/inspect_r6_rig.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
source=Path(sys.argv[1]); result=module.inspect(source)
assert len(result['parts'])==6 and result['skinJointCount']==12
assert result['animationCount']==0
raw=source.read_bytes(); length=struct.unpack_from('<I',raw,12)[0]
data=json.loads(raw[20:20+length]); data['nodes'][result['parts']['Head']['node']]['name']='wrong'
payload=json.dumps(data).encode();payload+=b' '*((-len(payload))%4)
with tempfile.TemporaryDirectory() as temp:
 p=Path(temp)/'broken.glb';p.write_bytes(struct.pack('<4sIIII',b'glTF',2,20+len(payload),len(payload),0x4e4f534a)+payload)
 try: module.inspect(p)
 except ValueError: pass
 else: raise AssertionError('Missing joint accepted')
print('Downloaded R6 hierarchy and missing-joint rejection passed')
