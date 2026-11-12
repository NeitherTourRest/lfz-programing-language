"""T21 public byte oracles and behavioral comparisons for `lfz fmt FILE`."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    for name in ('binary', 'output', 'baseline'):
        parser.add_argument('--' + name, type=Path, required=name != 'baseline')
    parser.add_argument('--expected-sha', required=True)
    parser.add_argument('--baseline-sha')
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument('--prepare-only', action='store_true')
    modes.add_argument('--red-only', action='store_true')
    options = parser.parse_args()
    exe, output = options.binary.resolve(), options.output.resolve()
    baseline = options.baseline.resolve() if options.baseline else None
    assert all(path.is_relative_to(ROOT) for path in (exe, output) if path)
    assert baseline is None or baseline.is_relative_to(ROOT)
    assert not output.exists()
    work = ROOT / 'Temp/T21-public'
    work.mkdir(parents=True, exist_ok=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    fixtures = {}

    def fixture(name, raw, expected, semantic=False):
        path = work / name
        path.write_bytes(raw)
        fixtures[name] = (path, raw, expected, semantic)

    fixture('basic.lfz', b'#42\n\tlet x = 1 \t\n\tprint(x)\t', b'#42\n let x = 1\n print(x)', True)
    fixture('bom.lfz', b'\xef\xbb\xbf#42\r\n\t1\t\r\n', b'\xef\xbb\xbf#42\r\n 1\r\n')
    fixture('bom-no-preamble.txt', b'\xef\xbb\xbf\t1 \t', b'\xef\xbb\xbf 1')
    fixture('mixed.lfz', b'#42\n\t1 \t\r\t \n\t2 \r\n\t3 \n\t4 \r\t5\t',
            b'#42\n 1\r\t \n 2\r\n 3\n 4\r 5')
    fixture('white.lfz', b'#42\n\t  \r\n \t\n\t ', b'#42\n\r\n\n')
    fixture('empty.txt', b'', b'')
    fixture('comments.lfz', b'#42\n\t// tail \t\n\t/* head \t\r\n\tinside \t\r\nend */ \t\n',
            b'#42\n // tail \t\n /* head \t\r\n\tinside \t\r\nend */\n')
    fixture('same_line_comment.lfz', b'#42\n\tprint(1 /* comment */ + 2) \t\n',
            b'#42\n print(1 /* comment */ + 2)\n', True)
    fixture('literal.lfz', '#42\n\tprint("雪  \\t${ 1 + 2 } ") \t\n'.encode(),
            '#42\n print("雪  \\t${ 1 + 2 } ")\n'.encode(), True)
    fixture('format_spec.lfz', b'#42\n\tprint("${\"ab\":>3}:${7:05d}:\\${literal}") \t\n',
            b'#42\n print("${\"ab\":>3}:${7:05d}:\\${literal}")\n', True)
    fixture('pipe_dump.lfz', b'#42\n\tlet x = [1,  2] |> len  \t\n\t;; \t\n',
            b'#42\n let x = [1,  2] |> len\n ;;\n')
    fixture('position.lfz', b'#42\nfn bad() { \t\n\t1/0 \t\n}\n\tlet r = attempt(bad) \t\n\tprint(jsonStringify(r.error)) \t\n',
            b'#42\nfn bad() {\n 1/0\n}\n let r = attempt(bad)\n print(jsonStringify(r.error))\n', True)
    fixture('control_effect.lfz', b'#42\n\tif true { \t\n\t\twriteFile("effect.txt", "OK") \t\n}\n',
            b'#42\n if true {\n  writeFile("effect.txt", "OK")\n}\n', True)
    fixture('no_dependency.lfz', b'#42\n\timport m from "./missing-dependency.lfz" \t\n\tprint(m) \t\n',
            b'#42\n import m from "./missing-dependency.lfz"\n print(m)\n')

    failures = {'invalid_utf8.lfz': b'#42\n\xff', 'bad_preamble.lfz': b' #42\n1\n',
                'lex_error.lfz': b'#42\n@\n', 'parse_error.lfz': b'#42\nlet x =\n'}
    for name, raw in failures.items():
        (work / name).write_bytes(raw)
    (work / 'fmt').write_bytes(b'print("legacy")\n')
    legacy = [('bare_fmt', ['fmt']), ('json_bare_fmt', ['--json', 'fmt']),
              ('run_fmt', ['run', 'fmt']),
              ('argv_fmt', ['-e', 'print(jsonStringify(argv()))', '--', 'fmt', '--json'])]
    misuse = [('missing_file', ['fmt', 'absent.lfz']), ('json_fmt', ['--json', 'fmt', 'basic.lfz']),
              ('extra_arg', ['fmt', 'basic.lfz', '--', 'x']), ('dash_file', ['fmt', '-x']),
              ('empty_path', ['fmt', ''])]
    names = list(fixtures) + list(failures) + [name for name, _ in misuse + legacy]
    if options.prepare_only:
        result = {'schema': 'LFZ-T21-PREPARED-1', 'cases_total': len(names), 'cases': names, 'executed': False}
    else:
        binary_sha = digest(exe.read_bytes())
        assert binary_sha == options.expected_sha.lower()
        if baseline:
            assert options.baseline_sha and digest(baseline.read_bytes()) == options.baseline_sha.lower()
        rows = []

        def run(args, executable=exe):
            p = subprocess.run([str(executable), *args], cwd=work, capture_output=True, timeout=8)
            return {'args': args, 'exit': p.returncode, 'stdout_hex': p.stdout.hex(), 'stderr_hex': p.stderr.hex()}

        for name in (names[:1] if options.red_only else names):
            assert digest(exe.read_bytes()) == binary_sha
            row = {'name': name, 'checks': {}}
            checks = row['checks']
            try:
                if name in fixtures:
                    path, raw, expected, semantic = fixtures[name]
                    mtime = path.stat().st_mtime_ns
                    row['input_hex'], row['expected_hex'] = raw.hex(), expected.hex()
                    actual = row['first'] = run(['fmt', name])
                    checks['bytes'] = (actual['exit'], actual['stdout_hex'], actual['stderr_hex']) == (0, expected.hex(), '')
                    checks['input_unchanged'] = path.read_bytes() == raw and path.stat().st_mtime_ns == mtime
                    if checks['bytes']:
                        formatted = work / ('formatted-' + name)
                        formatted.write_bytes(bytes.fromhex(actual['stdout_hex']))
                        row['second'] = run(['fmt', formatted.name])
                        checks['idempotent'] = row['second']['exit'] == 0 and row['second']['stdout_hex'] == actual['stdout_hex'] and not row['second']['stderr_hex']
                        if semantic:
                            effect = work / 'effect.txt'
                            for key, candidate in [('original_run', name), ('formatted_run', formatted.name)]:
                                effect.unlink(missing_ok=True)
                                row[key] = run(['run', candidate])
                                row[key]['effect_hex'] = effect.read_bytes().hex() if effect.exists() else None
                            lhs, rhs = row['original_run'], row['formatted_run']
                            checks['same_behavior'] = lhs['exit'] == rhs['exit'] == 0 and all(lhs[k] == rhs[k] for k in ('stdout_hex', 'stderr_hex', 'effect_hex'))
                            if name == 'position.lfz':
                                payload = json.loads(bytes.fromhex(lhs['stdout_hex']))
                                checks['observable_position'] = payload['class'] == 'ZeroDivisionError' and payload['line'] == 3 and payload['col'] > 0 and bool(payload['traceback'])
                            if name == 'control_effect.lfz':
                                checks['write_effect'] = lhs['effect_hex'] == b'OK'.hex()
                elif name in dict(legacy):
                    args = dict(legacy)[name]
                    actual = row['first'] = run(args)
                    if baseline:
                        expected = row['legacy_baseline'] = run(args, baseline)
                        checks['legacy_exact'] = actual == expected
                    else:
                        checks['legacy'] = (actual['exit'] == 2 and (bool(actual['stderr_hex']) if name == 'bare_fmt' else b'IOError' in bytes.fromhex(actual['stdout_hex']))) if name in ('bare_fmt', 'json_bare_fmt') else (actual['exit'] == 0 and not actual['stderr_hex'] and bytes.fromhex(actual['stdout_hex']) == (b'legacy\n' if name == 'run_fmt' else b'["fmt","--json"]\n'))
                else:
                    actual = row['first'] = run(['fmt', name] if name in failures else dict(misuse)[name])
                    checks['failed_without_output'] = actual['exit'] == 2 and not actual['stdout_hex'] and bool(actual['stderr_hex'])
            except (subprocess.TimeoutExpired, ValueError, OSError, KeyError) as error:
                row['exception'] = repr(error)
                checks['completed'] = False
            row['passed'] = bool(checks) and all(checks.values())
            rows.append(row)
        assert digest(exe.read_bytes()) == binary_sha
        result = {'schema': 'LFZ-T21-CLI-1', 'binary_sha256': binary_sha,
                  'script_sha256': digest(Path(__file__).read_bytes()), 'checks_total': len(rows),
                  'checks_passed': sum(row['passed'] for row in rows), 'rows': rows}
    result['timestamp_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
    print(json.dumps({key: value for key, value in result.items() if key not in ('rows', 'cases')}))
    return 0 if options.prepare_only or result['checks_total'] == result['checks_passed'] else 1


if __name__ == '__main__':
    sys.exit(main())
