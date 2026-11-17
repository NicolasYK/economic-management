"""Extract a newly built ZIP and launch its packaged executable against disposable data."""
from pathlib import Path
import os
import platform
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
    result = subprocess.run([str(binary), '--data-dir', str(path / 'test-data'), '--smoke-test'],
                            capture_output=True, text=True, timeout=60)
    print(result.stdout)
    print(result.stderr)
    result.check_returncode()
    assert (path / 'test-data/finance.sqlite').is_file(), 'Packaged app did not initialize its test database'
print(f'Packaged {platform.system()} executable passed native smoke test')
