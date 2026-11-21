import hashlib
import json
from pathlib import Path
import subprocess


root = Path(__file__).resolve().parents[2]
output = Path(__file__).with_name('ai-guide.json')
binary = root / 'Temp/T29/candidate.exe'
expected = 'f0d483f7a0cbf2d9c45da5bd684327f8ac177ac4c2378e23230eba86d29a4ed9'
guide = root / 'docs/guide/ai/findIndex.md'
examples = [root / 'docs/guide/ai/examples' / name for name in ('05_find_index.lfz', '06_find_index_budget.lfz')]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save():
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf8')


assert not output.exists()
assert sha(binary) == expected
report = dict(binary=str(binary.relative_to(root)), binary_bytes=binary.stat().st_size, binary_sha256=expected, inputs={str(path.relative_to(root)): sha(path) for path in [guide, *examples]}, runs=[])
for example, budget in zip(examples, (False, True)):
    relative = str(example.relative_to(root))
    for mode in ('check', 'run'):
        assert sha(binary) == expected
        flags = (['--json', '--max-steps', '100'] if budget else ['--max-steps', '10000']) if mode == 'run' else []
        args = [mode, *flags, relative]
        result = subprocess.run([str(binary), *args], cwd=root, capture_output=True, timeout=10)
        stdout, stderr = result.stdout.decode('utf8'), result.stderr.decode('utf8')
        if mode == 'check':
            passed = result.returncode == 0 and not stdout and not stderr
        elif budget:
            try:
                passed = result.returncode == 2 and not stderr and json.loads(stdout)['error'] == 'ExecutionLimitError'
            except (ValueError, KeyError, TypeError):
                passed = False
        else:
            passed = result.returncode == 0 and stdout == 'findIndex witnesses passed\n' and not stderr
        report['runs'].append(dict(argv=args, exit=result.returncode, stdout=stdout, stderr=stderr, passed=passed))
        save()
        if mode == 'check' and not passed:
            break
report['inputs_unchanged'] = all(sha(root / name) == digest for name, digest in report['inputs'].items())
save()
print([(row['argv'][0], row['exit'], row['passed']) for row in report['runs']])
assert len(report['runs']) == 4 and all(row['passed'] for row in report['runs']) and report['inputs_unchanged']
