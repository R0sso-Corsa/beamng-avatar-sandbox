"""Generate an original, segmented R6-style test mesh. No dependencies."""
from pathlib import Path
import math
import argparse
import base64
import shutil
import struct

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'assets/source/noob_r6'
# Metres; X right, Y back, Z up. Standing on Z=0, facing -Y.
PARTS = [
    ('Torso', 'blue', (0, 0, .9), (.6, .3, .6)),
    ('Head', 'yellow', (0, 0, 1.35), (.375, .375, .3)),
    ('LeftArm', 'yellow', (.45, 0, .9), (.3, .3, .6)),
    ('RightArm', 'yellow', (-.45, 0, .9), (.3, .3, .6)),
    ('LeftLeg', 'green', (.15, 0, .3), (.3, .3, .6)),
    ('RightLeg', 'green', (-.15, 0, .3), (.3, .3, .6)),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--face-texture', type=Path, help='User-supplied Roblox face.png; creates a local private model variant')
    args = parser.parse_args()
    out = ROOT / 'assets/private/noob_r6' if args.face_texture else OUT
    if args.face_texture:
        texture = args.face_texture.read_bytes()
        if len(texture) < 24 or not texture.startswith(b'\x89PNG\r\n\x1a\n') or texture[12:16] != b'IHDR':
            parser.error('Face texture must be a PNG')
        width, height = struct.unpack('>II', texture[16:24])
        if not width or not height:
            parser.error('Face texture dimensions must be positive')
    out.mkdir(parents=True, exist_ok=True)
    if args.face_texture:
        shutil.copyfile(args.face_texture, out / 'face.png')
    lines = ['# Original R6-style noob test mesh; units: metres', 'mtllib noob_r6.mtl']
    count = 0

    def mesh(vertices, faces, material):
        nonlocal count
        lines.append('usemtl ' + material)
        lines.extend('v ' + ' '.join(f'{v:.6f}' for v in point) for point in vertices)
        lines.extend('f ' + ' '.join(str(count + i + 1) for i in face) for face in faces)
        count += len(vertices)

    # Outward-wound quads; importers may triangulate them.
    faces = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4),
             (3, 7, 6, 2), (0, 4, 7, 3), (1, 2, 6, 5)]
    corners = [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),
               (-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]
    for name, material, centre, size in PARTS:
        lines.extend(['o ' + name, 'g ' + name])
        vertices = [tuple(centre[i] + corner[i]*size[i]/2 for i in range(3))
                    for corner in corners]
        if name != 'Head':
            mesh(vertices, faces, material)
        if name == 'Head':
            # Classic-style cylinder with bevelled rims, not a sphere.
            segments = 64
            rings = [(1.2, .15), (1.225, .1875), (1.475, .1875), (1.5, .15)]
            head_vertices = [(r*math.cos(i*math.tau/segments),
                              r*math.sin(i*math.tau/segments), z)
                             for z,r in rings for i in range(segments)]
            head_faces = [tuple(reversed(range(segments))),
                          tuple(range(3*segments,4*segments))]
            for j in range(3):
                for i in range(segments):
                    k = (i+1) % segments
                    head_faces.append((j*segments+i,j*segments+k,
                                       (j+1)*segments+k,(j+1)*segments+i))
            mesh(head_vertices, head_faces, material)
            if args.face_texture:
                # Preserve the image's aspect ratio in front projection. Curved
                # strips follow the cylinder without stretching it across Head.
                scale = .25 / max(width, height)
                face_width, face_height = width*scale, height*scale
                strips = 32
                lines.append('usemtl roblox_face')
                panel = []
                for i in range(strips+1):
                    x = face_width*(i/strips-.5)
                    y = -math.sqrt(.1875**2-x*x)-.002
                    for v in (0,1):
                        panel.append((x,y,1.35+face_height*(v-.5)))
                        lines.append(f'vt {i/strips:.6f} {v}')
                lines.extend('v ' + ' '.join(f'{v:.6f}' for v in point) for point in panel)
                for i in range(strips):
                    indices = (2*i,2*i+2,2*i+3,2*i+1)
                    lines.append('f ' + ' '.join(f'{count+j+1}/{j+1}' for j in indices))
                count += len(panel)
                continue
            # Original smile geometry, grouped with Head so it follows the head.
            for x in (-.105, .105):
                ring = [(x+.022*math.cos(a),
                         -math.sqrt(.1875**2-(x+.022*math.cos(a))**2)-.002,
                         1.39+.031*math.sin(a))
                        for a in (i*math.tau/12 for i in range(12))]
                mesh(ring, [tuple(range(12))], 'face')
            outer = [(-.12+.24*i/12,
                      -math.sqrt(.1875**2-(-.12+.24*i/12)**2)-.002,
                      1.285+.055*((i-6)/6)**2)
                     for i in range(13)]
            inner = [(x,y,z+.014) for x,y,z in outer]
            mesh(outer+inner, [(i,i+1,i+14,i+13) for i in range(12)], 'face')

    (out / 'noob_r6.obj').write_text('\n'.join(lines)+'\n')
    colours = {'yellow': (1, .8, 0), 'blue': (.05, .35, .85),
               'green': (.2, .65, .15), 'face': (.025, .025, .025)}
    materials = '\n'.join(
        f'newmtl {name}\nKd {r} {g} {b}\nKa 0 0 0\nKs 0 0 0\nd 1\nillum 1\n'
        for name,(r,g,b) in colours.items())
    if args.face_texture:
        materials += '\nnewmtl roblox_face\nKd 1 1 1\nd 1\nillum 1\nmap_Kd face.png\nmap_d -imfchan a face.png\n'
    (out / 'noob_r6.mtl').write_text(materials)
    # Front-view reference matches the generated body dimensions and materials.
    svg = ['<svg xmlns="http://www.w3.org/2000/svg" width="640" height="720" viewBox="0 0 640 720">',
           '<rect width="640" height="720" fill="#18202b"/>',
           '<text x="320" y="62" text-anchor="middle" fill="white" font-family="sans-serif" font-size="25">R6-style noob test character</text>']
    for name, material, centre, size in PARTS:
        x = 320+(centre[0]-size[0]/2)*360
        y = 630-(centre[2]+size[2]/2)*360
        rgb = colours[material]
        colour = '#'+''.join(f'{round(v*255):02x}' for v in rgb)
        rounding = ' rx="13.5"' if name == 'Head' else ''
        svg.append(f'<rect x="{x}" y="{y}" width="{size[0]*360}" height="{size[2]*360}"{rounding} fill="{colour}" stroke="#18202b" stroke-width="2"/>')
    if args.face_texture:
        data = base64.b64encode(texture).decode('ascii')
        svg.append(f'<image x="{320-face_width*180}" y="{144-face_height*180}" width="{face_width*360}" height="{face_height*360}" preserveAspectRatio="xMidYMid meet" href="data:image/png;base64,{data}"/>')
    else:
        svg.extend(['<g fill="#060606"><ellipse cx="282.2" cy="129.6" rx="7.92" ry="11.16"/><ellipse cx="357.8" cy="129.6" rx="7.92" ry="11.16"/></g>',
                    '<path d="M276.8 147.6 Q320 187.2 363.2 147.6" fill="none" stroke="#060606" stroke-width="5"/>'])
    svg.extend(['<text x="320" y="678" text-anchor="middle" fill="#b8c5d5" font-family="sans-serif" font-size="16">Six separate parts · 1.5 m tall · front view</text>', '</svg>'])
    (out / 'preview.svg').write_text('\n'.join(svg)+'\n')
    print(out)


if __name__ == '__main__':
    main()
