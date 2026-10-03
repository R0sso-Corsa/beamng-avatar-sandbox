"""Check rounded head geometry and unstretched face UVs without a renderer."""
import importlib.util
from pathlib import Path
import struct
import sys
import tempfile
import zlib

spec = importlib.util.spec_from_file_location('model', Path(__file__).resolve().parents[1] / 'tools/create_noob_model.py')
model = importlib.util.module_from_spec(spec)
spec.loader.exec_module(model)


def chunk(kind, data):
    return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind+data))


with tempfile.TemporaryDirectory() as directory:
    model.ROOT = Path(directory)
    model.OUT = model.ROOT / 'source'
    sys.argv = ['model']
    model.main()
    source = (model.OUT / 'noob_r6.obj').read_text()
    assert source.count('\no ') == 6
    head = source.split('o Head\n')[1].split('\no ')[0]
    points = [list(map(float, l.split()[1:])) for l in head.splitlines() if l.startswith('v ')]
    assert abs(max(p[0] for p in points)-min(p[0] for p in points)-.375) < 1e-6
    assert len(points) > 200  # Round head, not the former eight-vertex box.
    for width, height in [(2, 2), (2, 1), (1, 2)]:
        png = (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0))
               + chunk(b'IDAT', zlib.compress((b'\0'+bytes([0,0,0,255])*width)*height)) + chunk(b'IEND', b''))
        texture = model.ROOT / 'test.png'
        texture.write_bytes(png)
        sys.argv = ['model', '--face-texture', str(texture)]
        model.main()
        folder = model.ROOT / 'assets/private/noob_r6'
        assert (folder/'face.png').read_bytes() == png
        obj = (folder/'noob_r6.obj').read_text()
        decal = obj.split('usemtl roblox_face\n')[1].split('\no ')[0]
        points = [list(map(float, l.split()[1:])) for l in decal.splitlines() if l.startswith('v ')]
        dx = max(p[0] for p in points)-min(p[0] for p in points)
        dz = max(p[2] for p in points)-min(p[2] for p in points)
        assert abs(dx/dz-width/height) < 1e-4
        assert abs(max(dx,dz)-.25) < 1e-6
        assert len(set(p[1] for p in points)) > 2
        assert 'preserveAspectRatio="xMidYMid meet"' in (folder/'preview.svg').read_text()
        vertices = [l for l in obj.splitlines() if l.startswith('v ')]
        for line in obj.splitlines():
            if line.startswith('f '):
                assert all(1 <= int(i.split('/')[0]) <= len(vertices) for i in line.split()[1:])
print('Rounded head and square/wide/tall face proportions passed')
