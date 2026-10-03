"""Generate an original, segmented R6-style test mesh. No dependencies."""
from pathlib import Path
import math
import argparse
import base64
import shutil

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'assets/source/noob_r6'
# Metres; X right, Y back, Z up. Standing on Z=0, facing -Y.
PARTS = [
    ('Torso', 'blue', (0, 0, .9), (.6, .3, .6)),
    ('Head', 'yellow', (0, 0, 1.35), (.6, .3, .3)),
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
        if not texture.startswith(b'\x89PNG\r\n\x1a\n'):
            parser.error('Face texture must be a PNG')
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
        mesh(vertices, faces, material)
        if name == 'Head':
            if args.face_texture:
                # Transparent decal panel, outward normal -Y; follows Head.
                lines.extend(['usemtl roblox_face',
                              'vt 0 0', 'vt 1 0', 'vt 1 1', 'vt 0 1'])
                panel = [(-.3,-.151,1.2),(.3,-.151,1.2),
                         (.3,-.151,1.5),(-.3,-.151,1.5)]
                lines.extend('v ' + ' '.join(f'{v:.6f}' for v in point) for point in panel)
                lines.append('f ' + ' '.join(f'{count+i+1}/{i+1}' for i in range(4)))
                count += 4
                continue
            # Original smile geometry, grouped with Head so it follows the head.
            for x in (-.105, .105):
                ring = [(x+.022*math.cos(a), -.151, 1.39+.031*math.sin(a))
                        for a in (i*math.tau/12 for i in range(12))]
                mesh(ring, [tuple(range(12))], 'face')
            outer = [(-.12+.24*i/12, -.151, 1.285+.055*((i-6)/6)**2)
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
    for _, material, centre, size in PARTS:
        x = 320+(centre[0]-size[0]/2)*360
        y = 630-(centre[2]+size[2]/2)*360
        rgb = colours[material]
        colour = '#'+''.join(f'{round(v*255):02x}' for v in rgb)
        svg.append(f'<rect x="{x}" y="{y}" width="{size[0]*360}" height="{size[2]*360}" fill="{colour}" stroke="#18202b" stroke-width="2"/>')
    if args.face_texture:
        data = base64.b64encode(texture).decode('ascii')
        svg.append(f'<image x="212" y="90" width="216" height="108" preserveAspectRatio="none" href="data:image/png;base64,{data}"/>')
    else:
        svg.extend(['<g fill="#060606"><ellipse cx="282.2" cy="129.6" rx="7.92" ry="11.16"/><ellipse cx="357.8" cy="129.6" rx="7.92" ry="11.16"/></g>',
                    '<path d="M276.8 147.6 Q320 187.2 363.2 147.6" fill="none" stroke="#060606" stroke-width="5"/>'])
    svg.extend(['<text x="320" y="678" text-anchor="middle" fill="#b8c5d5" font-family="sans-serif" font-size="16">Six separate parts · 1.5 m tall · front view</text>', '</svg>'])
    (out / 'preview.svg').write_text('\n'.join(svg)+'\n')
    print(out)


if __name__ == '__main__':
    main()
