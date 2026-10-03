"""Inspect a GLB's R6 skeleton without executing embedded scripts."""
import argparse
import hashlib
import json
import struct
from pathlib import Path

BONES = {'Torso': 'Torso_00', 'Head': 'Head_01', 'LeftArm': 'Left Arm_02',
         'RightArm': 'Right Arm_03', 'LeftLeg': 'Left Leg_04', 'RightLeg': 'Right Leg_05'}

def inspect(path):
    raw = Path(path).read_bytes()
    if len(raw) < 20 or len(raw) > 32*1024*1024:
        raise ValueError('Invalid GLB size')
    magic, version, size = struct.unpack_from('<4sII', raw)
    length, kind = struct.unpack_from('<II', raw, 12)
    if magic != b'glTF' or version != 2 or size != len(raw) or kind != 0x4e4f534a or length > len(raw)-20:
        raise ValueError('Invalid GLB header')
    data = json.loads(raw[20:20+length])
    nodes = data.get('nodes', [])
    joints = {i for skin in data.get('skins', []) for i in skin['joints']}
    parents = {}
    for parent, node in enumerate(nodes):
        for child in node.get('children', []):
            if child in parents or not 0 <= child < len(nodes):
                raise ValueError('Invalid node hierarchy')
            parents[child] = parent
    mapping = {}
    for part, name in BONES.items():
        matches = [i for i in joints if 0 <= i < len(nodes) and nodes[i].get('name') == name]
        if len(matches) != 1:
            raise ValueError('Missing or ambiguous joint: '+name)
        i = matches[0]
        mapping[part] = {'node': i, 'name': name, 'parentNode': parents.get(i),
                         'localBind': {k: nodes[i][k] for k in ('matrix','translation','rotation','scale') if k in nodes[i]}}
    torso = mapping['Torso']['node']
    if any(mapping[p]['parentNode'] != torso for p in BONES if p != 'Torso'):
        raise ValueError('Expected limbs and head directly below torso')
    return {'schemaVersion': 1, 'sha256': hashlib.sha256(raw).hexdigest(),
            'sourceCoordinates': 'glTF Y-up; preserve ancestor transforms',
            'parts': mapping, 'skinJointCount': len(joints),
            'animationCount': len(data.get('animations', [])),
            'status': 'Joint names and hierarchy verified; weights and deformation not yet verified'}

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('glb'); parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    result = json.dumps(inspect(args.glb), indent=2)+'\n'
    if args.output: args.output.write_text(result)
    else: print(result, end='')
