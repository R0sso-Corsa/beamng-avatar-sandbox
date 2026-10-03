"""Convert decoded Roblox XML pose tracks to JSON and typed Rust source.
Source axes/units are preserved: retargeting and metre conversion are separate.
"""
import argparse
import json
import math
import xml.etree.ElementTree as ET
from pathlib import Path

PARTS={'HumanoidRootPart','Torso','Head','Left Arm','Right Arm','Left Leg','Right Leg'}
def properties(item):
    return {p.get('name'):p for p in item.findall('./Properties/*')}
def value(props,name,default):
    p=props.get(name)
    return default if p is None else p.text

def quaternion(m):
    # Choose the largest component to stay stable near half turns.
    candidates=[1+m[0]-m[4]-m[8],1-m[0]+m[4]-m[8],1-m[0]-m[4]+m[8],1+m[0]+m[4]+m[8]]
    i=max(range(4),key=lambda j:candidates[j]);s=math.sqrt(max(candidates[i],0))*2
    if s<1e-8:raise ValueError('Invalid rotation')
    if i==3:q=[(m[7]-m[5])/s,(m[2]-m[6])/s,(m[3]-m[1])/s,s/4]
    elif i==0:q=[s/4,(m[1]+m[3])/s,(m[2]+m[6])/s,(m[7]-m[5])/s]
    elif i==1:q=[(m[1]+m[3])/s,s/4,(m[5]+m[7])/s,(m[2]-m[6])/s]
    else:q=[(m[2]+m[6])/s,(m[5]+m[7])/s,s/4,(m[3]-m[1])/s]
    norm=math.sqrt(sum(x*x for x in q));return [x/norm for x in q]

def convert(path):
    raw=Path(path).read_bytes()
    if len(raw)>32*1024*1024 or b'<!DOCTYPE' in raw or b'<!ENTITY' in raw:raise ValueError('Unsupported XML')
    root=ET.fromstring(raw); sequences=root.findall(".//Item[@class='KeyframeSequence']")
    if len(sequences)!=1:raise ValueError('Expected one sequence')
    sequence=sequences[0];tracks={}; times=[]
    for frame in sequence.findall("./Item[@class='Keyframe']"):
        time=float(value(properties(frame),'Time','0'))
        if not math.isfinite(time) or time<0:raise ValueError('Invalid key time')
        times.append(time)
        def visit(pose,parent):
            props=properties(pose);name=value(props,'Name','')
            if name not in PARTS:raise ValueError('Unknown R6 part '+name)
            path=parent+'/'+name
            cf=props.get('CFrame')
            if cf is None: numbers=[0,0,0,1,0,0,0,1,0,0,0,1]
            else:numbers=[float(cf.findtext(k)) for k in ['X','Y','Z','R00','R01','R02','R10','R11','R12','R20','R21','R22']]
            if not all(math.isfinite(x) for x in numbers):raise ValueError('Nonfinite pose')
            m=numbers[3:]
            for i in range(3):
                for j in range(3):
                    if abs(sum(m[i*3+k]*m[j*3+k] for k in range(3))-(i==j))>1e-3:raise ValueError('Nonorthogonal rotation')
            weight=float(value(props,'Weight','1'));style=int(value(props,'EasingStyle','0'))
            if weight!=1 or style!=0:raise ValueError('Unsupported weight/easing; export/bake in Studio')
            tracks.setdefault(path,[]).append({'time':time,'translation':numbers[:3], 'rotation':quaternion(m),
                'easingStyle':style,'easingDirection':int(value(props,'EasingDirection','0')),
                'easingDefaulted':'EasingStyle' not in props})
            for child in pose.findall("./Item[@class='Pose']"):visit(child,path)
        for pose in frame.findall("./Item[@class='Pose']"):visit(pose,'')
    if not times or not tracks:raise ValueError('Empty animation')
    for keys in tracks.values():
        keys.sort(key=lambda k:k['time'])
        if any(a['time']>=b['time'] for a,b in zip(keys,keys[1:])):raise ValueError('Duplicate key times')
    return {'schemaVersion':1,'name':Path(path).stem,'duration':max(times),
            'loop':value(properties(sequence),'Loop','false')=='true',
            'coordinates':'Roblox Y-up, studs; local Motor6D deltas, not Blender bone transforms',
            'tracks':tracks}

def rust_source(clip):
    text=['use avatar_physics::animation::{Track,Key,Pose,Interpolation};','fn tracks() -> Vec<Track> { vec![']
    for keys in clip['tracks'].values():
        text.append('Track::new(vec![')
        for k in keys:
            text.append('Key {time:'+repr(k['time'])+',pose:Pose {translation:'+repr(k['translation'])+',rotation:'+repr(k['rotation'])+'},interpolation:Interpolation::Linear},')
        text.append(']).unwrap(),')
    text+= ['] }','fn main() {',f'let duration={clip["duration"]!r};',
             'for track in tracks() { for i in 0..=240 { let pose=track.sample(duration*i as f64/240.0,None).unwrap(); assert!(pose.rotation.iter().all(|v| v.is_finite())); } }','}']
    return '\n'.join(text)+'\n'
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('xml',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args();clip=convert(args.xml)
    args.output.write_text(json.dumps(clip,indent=2)+'\n')
    args.output.with_suffix('.rs').write_text(rust_source(clip))
    print(clip['name'],len(clip['tracks']),'tracks',clip['duration'],'seconds')
