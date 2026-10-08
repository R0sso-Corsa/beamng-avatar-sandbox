"""Replay recorded native R6 traces through the Rust C ABI (no game needed).

Ground trials use the recorded 0.6s forward/reverse/stop schedule. One captured
frame of input latency is retained explicitly. Fits describe this fixture only.
"""
import argparse
import ctypes as C
import json
import math
import platform
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
class Profile(C.Structure):
    _fields_ = [(n, C.c_double) for n in ('metres_per_stud gravity walk_speed jump_speed '
        'ground_acceleration air_acceleration mass radius height max_slope_degrees '
        'step_height recovery_distance static_friction dynamic_friction').split()]
class State(C.Structure):
    _fields_ = [('position', C.c_double*3), ('velocity', C.c_double*3), ('grounded', C.c_uint32)]
class Triangle(C.Structure):
    _fields_ = [('id', C.c_uint64), ('vertices', (C.c_double*3)*3)]
class Camera(C.Structure):
    _fields_ = [('position',C.c_double*3), ('forward',C.c_double*3), ('first_person',C.c_uint32),
        ('heading',C.c_double), ('requested_distance',C.c_double), ('actual_distance',C.c_double)]

def library():
    name = {'Darwin':'libavatar_physics.dylib','Linux':'libavatar_physics.so','Windows':'avatar_physics.dll'}[platform.system()]
    lib = C.CDLL(str(ROOT/'physics/target/release'/name))
    signatures = {
        'avatar_default_profile_v1':([C.POINTER(Profile)],C.c_int32),
        'avatar_create_with_profile_v1':([C.POINTER(Profile)]+[C.c_double]*3,C.c_uint64),
        'avatar_destroy':([C.c_uint64],C.c_int32),
        'avatar_set_mesh':([C.c_uint64,C.POINTER(Triangle),C.c_uint32],C.c_int32),
        'avatar_step':([C.c_uint64,C.c_double,C.c_double,C.c_uint32],C.c_int32),
        'avatar_get_state':([C.c_uint64,C.POINTER(State)],C.c_int32),
        'avatar_motor_response_v1':([C.c_uint64,C.c_double],C.c_int32),
        'avatar_camera_active_v1':([C.c_uint64,C.c_uint32],C.c_int32),
        'avatar_camera_classic_v1':([C.c_uint64,C.c_double],C.c_int32),
        'avatar_camera_zoom_v1':([C.c_uint64,C.c_double],C.c_int32),
        'avatar_camera_distance_v1':([C.c_uint64,C.c_double],C.c_int32),
        'avatar_camera_advance_v1':([C.c_uint64,C.c_double,C.c_double],C.c_int32),
        'avatar_camera_pose_v1':([C.c_uint64,C.POINTER(C.c_double),C.POINTER(C.c_double),C.c_double,C.POINTER(Camera)],C.c_int32),
    }
    for name,(args,ret) in signatures.items():
        fn = getattr(lib,name); fn.argtypes=args; fn.restype=ret
    return lib

def rmse(errors):
    return math.sqrt(sum(e*e for e in errors)/len(errors))

def replay(lib, samples, acceleration=None, response=0, jump=False, jump_speed=None, magnitude=1, speed=16, reverse=True):
    p=Profile(); assert lib.avatar_default_profile_v1(C.byref(p))==0
    if acceleration is not None: p.ground_acceleration=acceleration*p.metres_per_stud
    if jump_speed is not None: p.jump_speed=jump_speed*p.metres_per_stud
    p.walk_speed=speed*p.metres_per_stud
    h=lib.avatar_create_with_profile_v1(C.byref(p),0,0,p.height/2+0.0001); assert h
    floor=Triangle(1,((C.c_double*3)*3)((-100,-100,0),(100,-100,0),(0,100,0)))
    errors=[]; state=State(); step=1/240; latency=samples[0]['t']; index=0
    try:
        assert lib.avatar_set_mesh(h,C.byref(floor),1)==0
        assert lib.avatar_motor_response_v1(h,response)==0
        # Establish contact without adding a hidden command/sample delay.
        assert lib.avatar_step(h,0,0,0)==0
        initial=p.height/2+0.0001; positions=[]; airborne_errors=[]
        for sample in samples:
            while (index+1)*step <= sample['t']+1e-7:
                command_time=index*step-latency
                movement=0 if command_time < -1e-7 or jump else magnitude if command_time < 0.6-1e-7 else -magnitude if reverse and command_time < 1.2-1e-7 else 0
                press=jump and index==round(latency/step)
                assert lib.avatar_step(h,movement,0,int(press))==0
                index+=1
            assert lib.avatar_get_state(h,C.byref(state))==0
            errors.append(state.velocity[2 if jump else 0]/p.metres_per_stud-sample['vy' if jump else 'v'])
            positions.append((state.position[2]-initial)/p.metres_per_stud)
            if jump and sample['state'] in ('Jumping','Freefall') and sample['y']-samples[0]['y']>0.25: airborne_errors.append(errors[-1])
        return {'velocity_rmse_studs_s':rmse(errors),'peak_rise_studs':max(positions), **({'airborne_velocity_rmse_studs_s':rmse(airborne_errors)} if airborne_errors else {})}
    finally: assert lib.avatar_destroy(h)==0

def compare(data):
    lib=library(); result={'metadata':data['metadata'], 'scope':'one recorded server-owned R6; fitted caps are not universal material constants', 'materials':[]}
    # Estimate response from unsaturated high-friction samples. The two fixed
    # steps per captured frame expose the discrete decay without source code.
    high=data['materials'][-1]['samples']; dt=high[2]['t']-high[1]['t']
    retention=(16-high[2]['v'])/(16-high[1]['v'])
    rate=(1-retention**((1/240)/dt))*240
    result['fitted_response_hz']=rate
    for trial in data['materials']:
        s=trial['samples']
        # High friction needs reversal to expose the cap; walking is unsaturated.
        i=next(i for i in range(1,len(s)) if s[i]['t']>0.61) if trial['friction']==1 else 1
        cap=abs(s[i]['v']-s[i-1]['v'])/(s[i]['t']-s[i-1]['t'])
        legacy=replay(lib,s)
        fitted=replay(lib,s,cap,rate)
        result['materials'].append({'friction':trial['friction'],'fitted_cap_studs_s2':cap,'legacy':legacy,'feedback':fitted})
    jump=next(t for t in data['jump'] if t['mode']=='power50')['samples']
    result['jump']={'native_peak_rise_studs':max(s['y'] for s in jump)-jump[0]['y'],'legacy':replay(lib,jump,jump=True), 'calibrated':replay(lib,jump,jump=True,jump_speed=50*1.06+data['metadata']['gravity']/240)}
    # Compare the two unobstructed spring transitions; property changes clamp
    # previous zoom before each native trial. Wheel/controller paths are separate.
    p=Profile(); assert lib.avatar_default_profile_v1(C.byref(p))==0
    h=lib.avatar_create_with_profile_v1(C.byref(p),0,0,1); assert h
    vec=(C.c_double*3)(0,0,0); pose=Camera(); result['camera']=[]
    try:
        assert lib.avatar_camera_classic_v1(h,p.metres_per_stud)==0
        assert lib.avatar_camera_active_v1(h,1)==0
        trials=data['camera']['trials']
        for index,start,target in [(3,1.01,0.99),(5,0.5,12.5)]:
            # Fully settle at the native trial's recorded initial distance.
            assert lib.avatar_camera_distance_v1(h,start*p.metres_per_stud)==0
            assert lib.avatar_camera_advance_v1(h,1,-1)==0
            assert lib.avatar_camera_distance_v1(h,target*p.metres_per_stud)==0
            errors=[]
            for s in trials[index]['samples']:
                assert lib.avatar_camera_advance_v1(h,s['dt'],-1)==0
                assert lib.avatar_camera_pose_v1(h,vec,vec,-1,C.byref(pose))==0
                errors.append(pose.actual_distance/p.metres_per_stud-s['distance'])
            result['camera'].append({'label':trials[index]['label'],'distance_rmse_studs':rmse(errors),'max_error_studs':max(map(abs,errors))})
        assert lib.avatar_camera_advance_v1(h,1/30,1.08)==0
        assert lib.avatar_camera_pose_v1(h,vec,vec,-1,C.byref(pose))==0 and pose.actual_distance<=1.08
        before=bytes(pose)
        assert lib.avatar_camera_advance_v1(h,float('nan'),-1)==-2
        assert lib.avatar_camera_pose_v1(h,vec,vec,-1,C.byref(pose))==0 and bytes(pose)==before
    finally: assert lib.avatar_destroy(h)==0
    if 'validation' in data:
        result['independent_validation']=[]
        caps={row['friction']:row['fitted_cap_studs_s2'] for row in result['materials']}
        for trial in data['validation']['trials']:
            samples=[{**s,'v':s['vx']} for s in trial['samples']]
            label=trial['label']
            if label.startswith('friction_'):
                r=replay(lib,samples,caps[float(label.split('_')[1])],rate)
            elif label.startswith('walk_'):
                _,speed,magnitude=label.split('_')
                r=replay(lib,samples,741.636,rate,speed=float(speed),magnitude=float(magnitude),reverse=False)
            elif label.startswith('power'):
                power=float(label[5:]); r=replay(lib,samples,jump=True,jump_speed=power*1.06+data['metadata']['gravity']/240)
            elif label=='height7.2':
                r=replay(lib,samples,jump=True,jump_speed=math.sqrt(2*data['metadata']['gravity']*7.2)+data['metadata']['gravity']/240)
            else: continue
            if label.startswith('power') or label.startswith('height'):
                r['native_peak_rise_studs']=max(s['y'] for s in samples)-samples[0]['y']
            result['independent_validation'].append({'label':label,**r})
    return result

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture',type=Path,default=ROOT/'tests/fixtures/native_r6_2026_10_06.json')
    parser.add_argument('--output',type=Path)
    args=parser.parse_args(); result=compare(json.loads(args.fixture.read_text()))
    text=json.dumps(result,indent=2)
    if args.output: args.output.write_text(text+'\n')
    print(text)
