import importlib.util
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from zipfile import ZipFile
root = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('avatar_package', root / 'tools/package_avatar.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
with TemporaryDirectory() as directory:
    p = Path(directory)
    for name in ['noob_r6.obj', 'noob_r6.mtl']:
        (p / name).write_bytes((root / 'assets/source/noob_r6' / name).read_bytes())
    manifest = dict(version=1, rig='R6', units='metres', up='Z', mesh='noob_r6.obj', files=['noob_r6.obj','noob_r6.mtl'])
    (p / 'avatar.json').write_text(json.dumps(manifest))
    first = m.package(p, p / 'first.zip')
    assert first == m.package(p, p / 'second.zip')
    with ZipFile(p / 'first.zip') as archive:
        assert len(json.loads(archive.read('avatar.json'))['sha256']) == 2
    manifest['files'].append('../escape.png')
    (p / 'avatar.json').write_text(json.dumps(manifest))
    try:
        m.package(p, p / 'bad.zip')
        raise AssertionError('Traversal accepted')
    except ValueError:
        pass
print('Avatar source package, content identity and path rejection passed')
