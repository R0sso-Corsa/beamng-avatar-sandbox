"""Public offline checks. Requires Cargo, C compiler, Python+lupa and Node.
Private source-asset checks run only when --private-assets is supplied.
"""
import argparse
import os
import platform
import subprocess
import sys
import tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def run(*args):subprocess.run(args,cwd=ROOT,check=True)
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--private-assets',action='store_true');args=parser.parse_args()
 run('cargo','test','--manifest-path','physics/Cargo.toml')
 run('cargo','build','--release','--manifest-path','physics/Cargo.toml')
 run(sys.executable,'tests/native_reference_smoke.py')
 for name in ['avatar_catalogue','avatar_package','camera','diagnostics','driver','model','placement','telemetry']:
  run(sys.executable,f'tests/{name}_smoke.py')
 run(sys.executable,'-c','from lupa import LuaRuntime; LuaRuntime().execute(open("tests/smoke.lua").read())')
 run('node','tests/controls_ui_smoke.js')
 if platform.system()!='Windows':
  with tempfile.TemporaryDirectory() as tmp:
   executable=str(Path(tmp)/'avatar-c-host')
   run(os.environ.get('CC','cc'),'physics/examples/c_host.c','-Iphysics/include','-Lphysics/target/release',
       '-lavatar_physics',f'-Wl,-rpath,{ROOT}/physics/target/release','-lm','-o',executable)
   run(executable)
 else:print('External C host not run: compile c_host.c with your Windows C toolchain separately')
 if args.private_assets:
  run(sys.executable,'tests/r6_animation_smoke.py')
  run(sys.executable,'tests/native_animation_smoke.py')
  if (ROOT/'assets/private/r6_animator_capture.json').exists() and (ROOT/'assets/private/r6_climb_audit.json').exists():
   run(sys.executable,'tests/studio_animation_parity.py')
 run(sys.executable,'tools/package_mod.py')
 print('Offline validation passed. This does not establish game compatibility.')
