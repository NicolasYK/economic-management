"""Check tag/version agreement and emit the matching bilingual changelog entry."""
import os
from pathlib import Path
import tomllib
root = Path(__file__).resolve().parents[1]
version = tomllib.loads((root / 'Cargo.toml').read_text())['package']['version']
tag = os.environ.get('GITHUB_REF_NAME', f'v{version}')
if tag != f'v{version}':
    raise SystemExit(f'Tag {tag!r} does not match Cargo.toml v{version}')
changelog = (root / 'CHANGELOG.md').read_text(encoding='utf-8')
marker = f'## [{version}]'
if marker not in changelog:
    raise SystemExit('Missing release changelog')
entry = changelog.split(marker, 1)[1].split('\n## [', 1)[0]
destination = root / 'dist/release-notes.md'
destination.parent.mkdir(exist_ok=True)
destination.write_text(marker + entry, encoding='utf-8')
