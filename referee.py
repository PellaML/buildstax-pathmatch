"""Kill semantic mutants in an isolated copy; compile errors do not count."""
from pathlib import Path
import os
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent
MUTANTS = [
    ('glob', "return !p.contains(&b'/')", 'return true'),
    ('glob', "Token::One => row[j] && b != b'/'", 'Token::One => row[j]'),
    ('glob', "reachable && b == b'/'", 'reachable'),
    ('glob', 'b <= hi) != *n', 'b <= hi) == *n'),
    ('ignore', "let anchored = s.starts_with('/')", 'let anchored = false'),
    ('ignore', 'if matches!(decision, Decision::Ignore { .. })', 'if matches!(decision, Decision::Include { .. })'),
]

with tempfile.TemporaryDirectory() as tmp:
    package = Path(tmp) / 'package'
    shutil.copytree(ROOT / 'package', package, ignore=shutil.ignore_patterns('target'))
    env = dict(os.environ, CARGO_TARGET_DIR=str(Path(tmp) / 'target'))

    def run():
        return subprocess.run(['cargo', 'test', '--offline', '--test', 'integration'],
                              cwd=package, env=env, capture_output=True, text=True)

    baseline = run()
    if baseline.returncode:
        raise SystemExit(baseline.stdout + baseline.stderr)
    for module, old, new in MUTANTS:
        path = package / 'src' / f'{module}.rs'
        original = path.read_text(encoding='utf-8')
        if original.count(old) != 1:
            raise SystemExit(f'invalid mutation anchor: {old}')
        path.write_text(original.replace(old, new), encoding='utf-8')
        result = run()
        path.write_text(original, encoding='utf-8')
        if result.returncode == 0 or 'test result: FAILED' not in result.stdout:
            raise SystemExit(f'not killed: {old}\n{result.stdout}{result.stderr}')
        print(f'killed: {old}')
    print(f'{len(MUTANTS)}/{len(MUTANTS)} killed by test failures')
