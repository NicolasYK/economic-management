"""Extract a newly built ZIP and launch its packaged executable against disposable data."""
from pathlib import Path
import os
import platform
import shutil
import subprocess
import tempfile
import zipfile
root = Path(__file__).resolve().parents[1]
system = 'windows' if os.name == 'nt' else 'macos'
archives = list((root / 'dist').glob(f'*-{system}-*.zip'))
if len(archives) != 1:
    raise SystemExit('Expected exactly one native ZIP package')
with tempfile.TemporaryDirectory(prefix='economic-package-smoke-') as folder:
    path = Path(folder)
    with zipfile.ZipFile(archives[0]) as archive:
        archive.extractall(path)
    binary = path / ('economic-management.exe' if system == 'windows' else 'Economic Management.app/Contents/MacOS/economic-management')
    if system == 'macos':
        binary.chmod(0o755)
    environment = os.environ.copy()
    if system == 'windows' and environment.get('MESA_DIRECTORY'):
        # Hosted runners lack OpenGL 2+. Add a verified software driver only to
        # the disposable test extraction; the published ZIP stays unchanged.
        drivers = Path(environment['MESA_DIRECTORY'])
        if not (drivers / 'opengl32.dll').is_file():
            raise RuntimeError('Missing configured Mesa OpenGL driver')
        for library in drivers.glob('*.dll'):
            shutil.copy2(library, binary.parent / library.name)
        environment['GALLIUM_DRIVER'] = 'llvmpipe'
    result = subprocess.run([str(binary), '--data-dir', str(path / 'test-data'), '--smoke-test'],
                            env=environment, capture_output=True, text=True, timeout=60)
    print(result.stdout)
    print(result.stderr)
    result.check_returncode()
    assert (path / 'test-data/finance.sqlite').is_file(), 'Packaged app did not initialize its test database'
print(f'Packaged {platform.system()} executable passed native smoke test')
