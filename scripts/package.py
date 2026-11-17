"""Package already-built binaries. Build-time Python only; no Python runtime shipped."""
import argparse
import hashlib
import os
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
VERSION = tomllib.loads((ROOT / "Cargo.toml").read_text())['package']['version']
DIST = ROOT / "dist"
TOOLS = ROOT / ".tools"


def download(name, url, checksum):
    TOOLS.mkdir(exist_ok=True)
    destination = TOOLS / name
    if not destination.exists():
        with urllib.request.urlopen(url, timeout=120) as response:
            destination.write_bytes(response.read())
    if hashlib.sha256(destination.read_bytes()).hexdigest() != checksum:
        raise RuntimeError(f"Checksum mismatch: {name}; refusing to execute")
    destination.chmod(0o755)
    return destination


def readme_files(directory):
    for name in ["README.md", "LICENSE", "CHANGELOG.md", "SECURITY.md"]:
        shutil.copy2(ROOT / name, directory / name)


def archive_zip(directory, output):
    with zipfile.ZipFile(output, 'w', zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(directory.rglob('*')):
            if path.is_file():
                archive.write(path, path.relative_to(directory))


def linux(binary):
    tool = download('appimagetool-1.9.1.AppImage',
        'https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage',
        'ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0')
    runtime = download('runtime-x86_64-pinned',
        'https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-x86_64',
        '617aa77314d57c8b8e4fae9c3f0b35af7b30d5529cb69e98f64f92510ed98a46')
    with tempfile.TemporaryDirectory(prefix='economic-appimage-', dir=TOOLS) as temporary:
        work = Path(temporary)
        app = work / 'EconomicManagement.AppDir'
        (app / 'usr/bin').mkdir(parents=True)
        lib = app / 'usr/lib'
        lib.mkdir()
        shutil.copy2(binary, app / 'usr/bin/economic-management')
        # Libraries loaded dynamically by winit are not all visible in the executable's ldd output.
        queue = [binary]
        for name in ['libX11.so.6', 'libXcursor.so.1', 'libXrandr.so.2', 'libXi.so.6',
                     'libXinerama.so.1', 'libxkbcommon.so.0', 'libxkbcommon-x11.so.0', 'libwayland-client.so.0',
                     'libwayland-cursor.so.0', 'libwayland-egl.so.1']:
            candidate = Path('/usr/lib/x86_64-linux-gnu') / name
            if not candidate.exists():
                raise RuntimeError(f'Missing required runtime library: {name}')
            shutil.copy2(candidate, lib / name)
            queue.append(candidate)
        seen = set()
        excluded = ('libc.so.', 'libm.so.', 'libpthread.so.', 'libdl.so.', 'librt.so.', 'ld-linux', 'libGL', 'libEGL')
        while queue:
            path = queue.pop()
            if str(path) in seen:
                continue
            seen.add(str(path))
            output = subprocess.check_output(['ldd', str(path)], text=True)
            for line in output.splitlines():
                if ' => /' not in line:
                    continue
                dependency = Path(line.split(' => ', 1)[1].split(' (', 1)[0])
                if dependency.name.startswith(excluded):
                    continue
                if not (lib / dependency.name).exists():
                    shutil.copy2(dependency, lib / dependency.name)
                    queue.append(dependency)
        launcher = app / 'AppRun'
        launcher.write_text('#!/bin/sh\nHERE="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"\nexport LD_LIBRARY_PATH="$HERE/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"\nexec "$HERE/usr/bin/economic-management" "$@"\n')
        launcher.chmod(0o755)
        (app / 'economic-management.desktop').write_text('[Desktop Entry]\nType=Application\nName=Economic Management\nExec=economic-management\nIcon=economic-management\nCategories=Office;Finance;\nTerminal=false\n')
        shutil.copy2(ROOT / 'assets/economic-management.svg', app / 'economic-management.svg')
        readme_files(app)
        subprocess.run([str(tool), '--appimage-extract'], cwd=work, check=True, stdout=subprocess.DEVNULL)
        output = DIST / f'economic-management-{VERSION}-linux-x86_64.AppImage'
        subprocess.run([str(work / 'squashfs-root/AppRun'), '--runtime-file', str(runtime), str(app), str(output)],
                       env={**os.environ, 'ARCH': 'x86_64'}, check=True)
        return output


def package(target, binary):
    DIST.mkdir(exist_ok=True)
    TOOLS.mkdir(exist_ok=True)
    if target == 'linux':
        return linux(binary)
    with tempfile.TemporaryDirectory(prefix='economic-package-', dir=TOOLS) as temporary:
        work = Path(temporary)
        readme_files(work)
        if target == 'windows':
            shutil.copy2(binary, work / 'economic-management.exe')
            suffix = 'windows-x86_64'
        else:
            contents = work / 'Economic Management.app/Contents'
            (contents / 'MacOS').mkdir(parents=True)
            shutil.copy2(binary, contents / 'MacOS/economic-management')
            with (contents / 'Info.plist').open('wb') as file:
                plistlib.dump(dict(CFBundleName='Economic Management', CFBundleDisplayName='Economic Management',
                    CFBundleIdentifier='io.github.nicolasyk.economic-management', CFBundleExecutable='economic-management',
                    CFBundlePackageType='APPL', CFBundleShortVersionString=VERSION, CFBundleVersion=VERSION,
                    NSHighResolutionCapable=True, LSMinimumSystemVersion='14.0'), file)
            suffix = f'macos-{platform.machine()}'
        output = DIST / f'economic-management-{VERSION}-{suffix}.zip'
        archive_zip(work, output)
        return output


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('target', choices=['linux', 'windows', 'macos'])
    parser.add_argument('--binary', type=Path)
    options = parser.parse_args()
    binary = options.binary or ROOT / 'target/release' / ('economic-management.exe' if options.target == 'windows' else 'economic-management')
    output = package(options.target, binary.resolve())
    checksum = hashlib.sha256(output.read_bytes()).hexdigest()
    output.with_name(output.name + '.sha256').write_text(f'{checksum}  {output.name}\n')
    print(output)
