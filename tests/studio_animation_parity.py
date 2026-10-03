"""Compare Rust C-ABI sampling to captured Studio Animator joint transforms."""
import ctypes as C
import importlib.util
import json
import math
import platform
from pathlib import Path
root=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('conversion',root/'tools/convert_r6_animation.py')
conversion=importlib.util.module_from_spec(spec);spec.loader.exec_module(conversion)
class Pose(C.Structure):_fields_=[('translation',C.c_double*3),('rotation',C.c_double*4)]
class Key(C.Structure):_fields_=[('time',C.c_double),('pose',Pose),('interpolation',C.c_uint32)]
name={'Darwin':'libavatar_physics.dylib','Linux':'libavatar_physics.so','Windows':'avatar_physics.dll'}[platform.system()]
lib=C.CDLL(str(root/'physics/target/release'/name))
lib.avatar_create.argtypes=[C.c_double]*3;lib.avatar_create.restype=C.c_uint64
lib.avatar_destroy.argtypes=[C.c_uint64]
lib.avatar_track_upload_v1.argtypes=[C.c_uint64,C.c_uint64,C.POINTER(Key),C.c_uint32]
lib.avatar_track_sample_v1.argtypes=[C.c_uint64,C.c_uint64,C.c_double,C.c_double,C.POINTER(Pose)]
capture=json.loads((root/'assets/private/r6_animator_capture.json').read_text())
dense=json.loads((root/'assets/private/r6_climb_audit.json').read_text())
capture['clips'].append({'name':'climb','scope':'dense left arm','samples':[{'time':s['time'],'poses':{'Left Arm':s['animator']}} for s in dense['samples']]})
handle=lib.avatar_create(0,0,1);assert handle
report={'studioVersion':capture['studioVersion'],'method':capture['method'],'clips':[]}
try:
 for source in capture['clips']:
  clip=json.loads((root/f'assets/private/r6_animations/{source["name"]}.json').read_text())
  maximum_translation=maximum_angle=0.0;count=0
  for index,(path,keys) in enumerate(clip['tracks'].items()):
   part=path.split('/')[-1]
   if part=='HumanoidRootPart':continue
   array=(Key*len(keys))(*[Key(k['time'],Pose((C.c_double*3)(*k['translation']),(C.c_double*4)(*k['rotation'])),2) for k in keys])
   assert lib.avatar_track_upload_v1(handle,index,array,len(keys))==0
   for sample in source['samples']:
    if part not in sample['poses']:continue
    actual=sample['poses'][part];pose=Pose()
    assert lib.avatar_track_sample_v1(handle,index,sample['time'],0,C.byref(pose))==0
    q=conversion.quaternion(actual[3:]);dot=abs(sum(a*b for a,b in zip(q,pose.rotation)))
    angle=2*math.acos(min(1.0,dot))
    error=max(abs(a-b) for a,b in zip(actual[:3],pose.translation))
    maximum_translation=max(maximum_translation,error);maximum_angle=max(maximum_angle,angle);count+=1
  entry={'scope':source.get('scope','six joints'), 'name':source['name'],'samples':count,'maxTranslationErrorStuds':maximum_translation,'maxRotationErrorRadians':maximum_angle,
         'passesTolerance':maximum_translation<1e-4 and maximum_angle<1e-3}
  report['clips'].append(entry);print(entry)
finally:assert lib.avatar_destroy(handle)==0
(root/'assets/source/r6_rig/studio_parity_report.json').write_text(json.dumps(report,indent=2)+'\n')
assert all(c['passesTolerance'] for c in report['clips']), 'Studio comparison mismatch; inspect report before claiming parity'
