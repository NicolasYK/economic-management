"""Prove the cent-preservation regression test detects a deliberately inverted comparison."""
from pathlib import Path
import os
import subprocess
root = Path(__file__).resolve().parents[1]
source = root / 'src/domain.rs'
original = source.read_text()
condition = 'i64::from(index) < total % i64::from(count)'
if original.count(condition) != 1:
    raise SystemExit('Mutation target changed; review this test before running it')
try:
    source.write_text(original.replace(condition, condition.replace(' < ', ' > ')))
    result = subprocess.run(['cargo', 'test', '--no-default-features', '--test', 'domain'], cwd=root,
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if result.returncode == 0 or 'installments_never_lose_a_cent ... FAILED' not in result.stdout:
        raise SystemExit('Mutation was not detected as the expected assertion failure:\n' + result.stdout)
    print('PASS: cent-preservation test rejected the inverted rounding condition')
finally:
    source.write_text(original)
