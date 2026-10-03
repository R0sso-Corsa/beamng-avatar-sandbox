"""Validate/package local R6 OBJ sources. Not a BeamNG mesh converter."""
import argparse
import hashlib
import json
import math
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED

PARTS = {'Head', 'Torso', 'LeftArm', 'RightArm', 'LeftLeg', 'RightLeg'}


def package(source, output):
    source = Path(source).resolve()
    manifest = json.loads((source / 'avatar.json').read_text())
    if manifest.get('version') != 1 or manifest.get('rig') != 'R6' or manifest.get('units') != 'metres' or manifest.get('up') != 'Z':
        raise ValueError('Expected version 1, R6, metres, Z-up')
    files = manifest.get('files')
    if not isinstance(files, list) or not files or len(files) > 32 or len(set(files)) != len(files):
        raise ValueError('Expected 1–32 unique file names')
    payload = {}
    for name in files:
        if not isinstance(name, str) or Path(name).name != name or '\\' in name or name in {'.', '..', 'avatar.json'}:
            raise ValueError('Files must have simple local names')
        path = source / name
        if path.is_symlink() or path.suffix.lower() not in {'.obj', '.mtl', '.png', '.jpg', '.jpeg'}:
            raise ValueError('Unsupported file or symlink')
        if path.stat().st_size > 8 * 1024 * 1024:
            raise ValueError('File exceeds 8 MiB')
        payload[name] = path.read_bytes()
    if sum(map(len, payload.values())) > 32 * 1024 * 1024:
        raise ValueError('Package exceeds 32 MiB')
    mesh = manifest.get('mesh')
    if mesh not in payload or not mesh.endswith('.obj'):
        raise ValueError('Mesh must be a listed OBJ')
    groups, vertices = set(), 0
    for line in payload[mesh].decode('utf8').splitlines():
        words = line.split()
        if not words:
            continue
        if words[0] in {'g', 'o'}:
            groups.update(words[1:])
        if words[0] == 'v':
            coords = list(map(float, words[1:4]))
            if len(coords) != 3 or any(not math.isfinite(v) or abs(v) > 100 for v in coords):
                raise ValueError('Invalid vertex')
            vertices += 1
        if words[0] == 'mtllib' and any(name not in payload for name in words[1:]):
            raise ValueError('Missing material file')
    if not PARTS <= groups or not 0 < vertices <= 100000:
        raise ValueError('Mesh requires six named R6 parts and 1–100000 vertices')
    for name, data in payload.items():
        if name.endswith('.mtl'):
            for line in data.decode('utf8').splitlines():
                words = line.split()
                if words and words[0].lower().startswith('map_') and (len(words) != 2 or words[1] not in payload):
                    raise ValueError('Texture references must name one listed local file')
    manifest['sha256'] = {name: hashlib.sha256(data).hexdigest() for name, data in sorted(payload.items())}
    canonical = json.dumps(manifest, sort_keys=True, separators=(',', ':')).encode()
    pack_id = hashlib.sha256(canonical).hexdigest()
    output = Path(output)
    output.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(output, 'w', ZIP_DEFLATED) as archive:
        archive.writestr('avatar.json', canonical)
        for name, data in sorted(payload.items()):
            archive.writestr(name, data)
    return pack_id


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('source')
    parser.add_argument('output')
    args = parser.parse_args()
    print(package(args.source, args.output))
