"""Convert local originals and exercise every track through Rust sampling."""
import importlib.util
import subprocess
import tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('convert',root/'tools/convert_r6_animation.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
subprocess.run(['cargo','build','--manifest-path',str(root/'physics/Cargo.toml')],check=True)
with tempfile.TemporaryDirectory() as temp:
 temp=Path(temp)
 for name in ['idle1','idle2','walk','jump','fall','climb']:
  clip=module.convert(root/f'assets/private/r6_animations/{name}.rbxmx')
  assert clip['duration']>0 and len(clip['tracks'])>=7
  assert all(path.split('/')[-1] in module.PARTS for path in clip['tracks'])
  source=temp/f'{name}.rs';source.write_text(module.rust_source(clip))
  binary=temp/name
  subprocess.run(['rustc','--edition=2021',str(source),'--extern',f'avatar_physics={root}/physics/target/debug/libavatar_physics.rlib','-L',f'dependency={root}/physics/target/debug/deps','-o',str(binary)],check=True)
  subprocess.run([str(binary)],check=True)
 invalid=temp/'invalid.xml';invalid.write_text('<!DOCTYPE x><roblox/>')
 try:module.convert(invalid)
 except ValueError:pass
 else:raise AssertionError('Unsupported XML accepted')
 assert module.quaternion([1,0,0,0,-1,0,0,0,-1])==[1.0,0.0,0.0,0.0]
print('Six converted source clips sampled in Rust; invalid XML rejected')
