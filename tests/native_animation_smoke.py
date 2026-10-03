"""Upload all six local clips through the public C ABI and verify key poses."""
import ctypes as C
import json
import math
import platform
from pathlib import Path
root=Path(__file__).resolve().parents[1]
ext={'Darwin':'dylib','Linux':'so','Windows':'dll'}[platform.system()]
name=('' if ext=='dll' else 'lib')+'avatar_physics.'+ext
lib=C.CDLL(str(root/'physics/target/release'/name))
class Pose(C.Structure):_fields_=[('translation',C.c_double*3),('rotation',C.c_double*4)]
class Key(C.Structure):_fields_=[('time',C.c_double),('pose',Pose),('interpolation',C.c_uint32)]
lib.avatar_create.argtypes=[C.c_double]*3;lib.avatar_create.restype=C.c_uint64
lib.avatar_destroy.argtypes=[C.c_uint64];lib.avatar_destroy.restype=C.c_int32
lib.avatar_track_upload_v1.argtypes=[C.c_uint64,C.c_uint64,C.POINTER(Key),C.c_uint32];lib.avatar_track_upload_v1.restype=C.c_int32
lib.avatar_track_sample_v1.argtypes=[C.c_uint64,C.c_uint64,C.c_double,C.c_double,C.POINTER(Pose)];lib.avatar_track_sample_v1.restype=C.c_int32
handle=lib.avatar_create(0,0,1);assert handle
try:
 for name in ['idle1','idle2','walk','jump','fall','climb']:
  clip=json.loads((root/f'assets/private/r6_animations/{name}.json').read_text())
  for index,keys in enumerate(clip['tracks'].values()):
   array=(Key*len(keys))(*[Key(k['time'],Pose((C.c_double*3)(*k['translation']),(C.c_double*4)(*k['rotation'])),2) for k in keys])
   assert lib.avatar_track_upload_v1(handle,index,array,len(keys))==0
   for key in keys:
    out=Pose();assert lib.avatar_track_sample_v1(handle,index,key['time'],0,C.byref(out))==0
    assert max(abs(a-b) for a,b in zip(out.translation,key['translation']))<1e-9
    assert abs(abs(sum(a*b for a,b in zip(out.rotation,key['rotation'])))-1)<1e-8
   for i in range(241):
    assert lib.avatar_track_sample_v1(handle,index,clip['duration']*i/240,clip['duration'] if clip['loop'] else 0,C.byref(out))==0
    assert all(math.isfinite(v) for v in out.translation)
  print(name,'source keys and loop/intermediate samples passed through native ABI')
finally:assert lib.avatar_destroy(handle)==0
