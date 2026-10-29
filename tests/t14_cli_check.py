"""T14 public-process check acceptance; only stdlib, project-local fixtures/evidence.

Each row is one process case with independent public-behavior expectations.
No interpreter source or third-party package is imported.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
CHECK_KEYS = ['ok', 'mode', 'executed']
ERROR_KEYS = CHECK_KEYS + ['error', 'message', 'file', 'line', 'col', 'traceback']


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def lit(value):
    return '"' + value.replace('\\', '\\\\').replace('"', '\\"').replace('$', '\\$').replace('\n', '\\n').replace('\r', '\\r').replace('\t', '\\t') + '"'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--binary', type=Path, required=True)
    ap.add_argument('--output', type=Path, required=True)
    ap.add_argument('--smoke-only', action='store_true', help='Run one expected new-command success case, suitable for a preimplementation red witness.')
    options = ap.parse_args()
    binary = (ROOT / options.binary).resolve()
    output = (ROOT / options.output).resolve()
    if not binary.is_relative_to(ROOT) or not output.is_relative_to(ROOT) or output.exists():
        ap.error('binary/output must be project-local and output must not exist')
    work = ROOT / 'Temp/T14-test'
    work.mkdir(parents=True, exist_ok=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env.update(TEMP=str(work), TMP=str(work), PYTHONDONTWRITEBYTECODE='1')
    rows = []
    initial_binary_hash = sha(binary)

    def fixture(name, data):
        path = work / name
        path.write_bytes(data if isinstance(data, bytes) else data.encode('utf-8'))
        return path

    def run(name, args, stdin=b'', *, mode='success', error=None, file=None, line=None, col=None,
            message=None, guards=None, expected_stdout=None, expected_exit=None):
        start = time.perf_counter()
        expected = {'mode': mode, 'error': error, 'file': file, 'line': line, 'col': col,
                    'message': message, 'stdout': expected_stdout, 'exit': expected_exit}
        checks = {}
        row = {'name': name, 'args': args, 'stdin_hex': stdin.hex(), 'expected': expected}
        try:
            assert sha(binary) == initial_binary_hash, 'binary changed during test'
            process = subprocess.run([str(binary), *args], input=stdin, capture_output=True,
                                     cwd=ROOT, env=env, timeout=5)
            stdout, stderr = process.stdout.decode('utf-8'), process.stderr.decode('utf-8')
            row.update(exit=process.returncode, stdout=stdout, stderr=stderr)
            json_mode = mode.startswith('json_')
            if mode in ('success', 'json_success'):
                checks['exit'] = process.returncode == 0
                checks['stderr_empty'] = stderr == ''
                if json_mode:
                    checks['success_exact_bytes'] = process.stdout == b'{"ok":true,"mode":"check","executed":false}\n'
                else:
                    checks['stdout_empty'] = stdout == ''
            elif mode in ('error', 'json_error'):
                checks['exit'] = process.returncode == 2
                if json_mode:
                    checks['stderr_empty'] = stderr == ''
                    try:
                        obj = json.loads(stdout)
                        checks['one_json_line'] = len(stdout.splitlines()) == 1
                        checks['ordered_keys'] = list(obj) == ERROR_KEYS
                        checks['check_markers'] = obj.get('ok') is False and obj.get('mode') == 'check' and obj.get('executed') is False
                        checks['error_class'] = obj.get('error') == error
                        if file is not None:
                            checks['error_file'] = obj.get('file') == file
                        if line is not None:
                            checks['error_line'] = obj.get('line') == line
                        if col is not None:
                            checks['error_col'] = obj.get('col') == col
                        if message is not None:
                            checks['error_message'] = obj.get('message') == message
                        if obj.get('line') is None:
                            checks['no_location_trace'] = obj.get('col') is None and obj.get('traceback') == []
                        else:
                            checks['location_trace'] = obj.get('traceback') == [{'file': obj.get('file'), 'line': obj.get('line'), 'func': '<module>'}]
                    except (ValueError, TypeError, AttributeError):
                        checks['parse_json'] = False
                else:
                    checks['stdout_empty'] = stdout == ''
                    checks['human_class'] = error + ':' in stderr
            elif mode == 'usage':
                checks['exit'] = process.returncode == 2
                checks['stdout_empty'] = stdout == ''
                checks['usage_text'] = stderr.startswith('lfz: ') and '用法' in stderr
            elif mode == 'control':
                checks['exit'] = process.returncode == expected_exit
                checks['stdout'] = stdout == expected_stdout
                checks['stderr_empty'] = stderr == ''
            else:
                raise AssertionError('unknown test mode')
        except subprocess.TimeoutExpired as exc:
            row.update(exit=None, stdout=(exc.stdout or b'').decode('utf-8', 'replace'), stderr=(exc.stderr or b'').decode('utf-8', 'replace'), exception='timeout')
            checks['returned_without_executing_infinite_or_input'] = False
        except Exception as exc:
            row['exception'] = repr(exc)
            checks['process'] = False
        for path, before in (guards or {}).items():
            after = path.read_bytes() if path.exists() else None
            checks['unchanged_' + path.name] = after == before
            row.setdefault('guard_evidence', []).append({'path': str(path.relative_to(ROOT)), 'before_hex': before.hex() if before is not None else None,
                'after_hex': after.hex() if after is not None else None})
        row.update(checks=checks, passed=all(checks.values()), elapsed_seconds=time.perf_counter() - start)
        rows.append(row)
        return row

    run('new_check_smoke', ['check', '-e', 'print("must-not-print")'], mode='success')
    if not options.smoke_only:
        valid = 'print("should-not-run")\neprint("should-not-run")\n;;\ninput("should-not-prompt")\n'
        valid_file = fixture('valid.lfz', '#42\n' + valid)
        for source, args, stdin in [('file', [str(valid_file)], b''), ('inline', ['-e', valid], b''), ('stdin', ['-'], valid.encode('utf-8'))]:
            run('sources_' + source, ['check', *args], stdin)
            run('sources_' + source + '_json', ['--json', 'check', *args], stdin, mode='json_success')
        for name, args in [
            ('empty', ['-e', '']), ('empty_json', ['-e', '', '--json']),
            ('code_option', ['-e', '--json']), ('code_help', ['-e', '--help']),
            ('code_check', ['-e', 'check']), ('unicode', ['-e', 'print("中文🙂 --json --")']),
            ('json_before', ['--json', '-e', 'unknownName']),
            ('json_after', ['-e', 'unknownName', '--json']),
            ('json_repeat', ['--json', '-e', 'unknownName', '--json', '--json']),
        ]:
            json_mode = name in ('empty_json', 'json_before', 'json_after', 'json_repeat')
            run('code_protection_' + name, ['check', *args], mode='json_success' if json_mode else 'success')
        run('code_option_json_outer', ['--json', 'check', '-e', '--json'], mode='json_success')
        run('code_dashdash_preserved', ['check', '-e', '--', '--json'], mode='json_error', error='SyntaxError', file='<command>')
        for index, tail in enumerate([
            [], ['-e'], ['--help'], ['--unknown'], [str(valid_file), str(valid_file)],
            ['-e', '1', 'extra'], ['--'], ['--', 'x'], [str(valid_file), '--'],
            [str(valid_file), '--', 'arg'], ['-e', '1', '--'], ['-e', '1', '--', '--json'],
            ['-', '--'], ['-', '--', 'arg'],
        ]):
            for json_flag in ([], ['--json']):
                run(f'usage_{index}_{bool(json_flag)}', [*json_flag, 'check', *tail], mode='usage')

        for index, body in enumerate([
            'unknownName', '1 + true', 'print(1/0)', 'assert(false)', 'check(false)',
            'fn f(x)=>x\nf()', 'let x=1\nx=2', 'readFile("definitely-missing-t14")',
            'while true {}', 'fn forever()=>forever()\nforever()',
            'jsonParse("invalid json")', 'if 1 { print("type-error-if-executed") }',
        ]):
            run('non_static_' + str(index), ['check', '-e', body])
            run('non_static_json_' + str(index), ['check', '-e', body, '--json'], mode='json_success')

        for name, data in [
            ('header.LFZ', '#42\nunknownName'), ('bom.lfz', b'\xef\xbb\xbf#42\r\nunknownName\r\n'),
            ('carriage.lfz', '#42\runknownName\r'), ('plain.txt', 'unknownName'),
            ('no-extension', 'unknownName'), ('empty.txt', ''), ('empty-source.lfz', '#42\n'),
        ]:
            path = fixture(name, data)
            run('loader_ok_' + name, ['check', str(path), '--json'], mode='json_success')
        for name, data, error in [
            ('no-header.lfz', 'unknownName', 'CosmosAnswerError'),
            ('header-space.lfz', '#42 \nunknownName', 'CosmosAnswerError'),
            ('header-blank.lfz', '\n#42\nunknownName', 'CosmosAnswerError'),
            ('invalid-utf8.lfz', b'#42\n\xff', 'SyntaxError'),
        ]:
            path = fixture(name, data)
            run('loader_error_' + name, ['check', str(path), '--json'], mode='json_error', error=error, file=str(path))
        missing = work / 'missing-source.lfz'
        if missing.exists():
            raise RuntimeError('reserved missing source unexpectedly exists')
        run('missing_source', ['check', str(missing), '--json'], mode='json_error', error='IOError', file=str(missing))
        run('stdin_invalid_utf8', ['check', '-', '--json'], b'\xff', mode='json_error', error='IOError', file='<stdin>', message='无法读取标准输入')
        for source, line, col in [('let x =\n', 1, 8), ('\nlet x =\n', 2, 8)]:
            run('inline_location_' + str(line), ['check', '-e', source, '--json'], mode='json_error', error='SyntaxError', file='<command>', line=line, col=col)
        location_file = fixture('location.lfz', '#42\nlet x =\n')
        run('file_location', ['check', str(location_file), '--json'], mode='json_error', error='SyntaxError', file=str(location_file), line=2, col=8)
        run('stdin_location', ['check', '-', '--json'], b'let x =\n', mode='json_error', error='SyntaxError', file='<stdin>', line=1, col=8)
        for index, body in enumerate(['return 1', 'break', 'continue', 'fn f(){ let x = }', 'if false { let x = }', '1 = 2', 'let x = @']):
            run('syntax_error_' + str(index), ['check', '-e', body, '--json'], mode='json_error', error='SyntaxError', file='<command>')

        existing = fixture('sentinel.bin', b'\x00original\r\n\xff')
        created = work / 'must-not-create.bin'
        if created.exists():
            raise RuntimeError('reserved creation sentinel unexpectedly exists')
        guards = {existing: existing.read_bytes(), created: None}
        mutation = '\n'.join([
            'print("no-output")', 'eprint("no-output")', ';;',
            'writeFile(' + lit(str(existing)) + ', "overwritten")',
            'appendFile(' + lit(str(existing)) + ', "appended")',
            'writeFile(' + lit(str(created)) + ', "created")',
            'appendFile(' + lit(str(created)) + ', "created")',
            'input("no-prompt")', 'while true {}',
        ])
        for index, ending in enumerate(['', '\nlet broken =', '\n@']):
            src = mutation + ending
            path = fixture('guard-' + str(index) + '.lfz', '#42\n' + src)
            for form, args, stdin in [('file', [str(path)], b''), ('inline', ['-e', src], b''), ('stdin', ['-'], src.encode('utf-8'))]:
                for json_flag in ([], ['--json']):
                    is_error = bool(ending)
                    mode = ('json_error' if json_flag else 'error') if is_error else ('json_success' if json_flag else 'success')
                    run(f'non_execution_{index}_{form}_{bool(json_flag)}', ['check', *args, *json_flag], stdin, mode=mode, error='SyntaxError' if is_error else None, guards=guards)

        run('old_inline_execution', ['-e', 'check(true)\nprint("control")'], mode='control', expected_exit=0, expected_stdout='control\n')
        run('old_run_json_unchanged', ['-e', '1', '--json'], mode='control', expected_exit=0, expected_stdout='{"ok":true}\n')
        named_check = fixture('check', 'print("named-check")')
        run('old_run_named_check_file', ['run', str(named_check)], mode='control', expected_exit=0, expected_stdout='named-check\n')
        run('check_named_check_file', ['check', str(named_check), '--json'], mode='json_success')
        help_row = run('help_documents_check', ['--help'], mode='control', expected_exit=0, expected_stdout=None)
        help_row['checks']['stdout'] = 'lfz check' in help_row.get('stdout', '')
        help_row['passed'] = all(help_row['checks'].values())
    passed = sum(row['passed'] for row in rows)
    result = {'schema': 'LFZ-T14-CHECK-BLACKBOX-1', 'date': '2026-10-29',
              'timestamp_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'binary': str(binary.relative_to(ROOT)), 'binary_sha256': initial_binary_hash,
              'binary_bytes': binary.stat().st_size, 'script_sha256': sha(Path(__file__)),
              'smoke_only': options.smoke_only, 'total': len(rows), 'passed': passed,
              'failed': len(rows) - passed, 'rows': rows}
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
    print(json.dumps({key: value for key, value in result.items() if key != 'rows'}, ensure_ascii=False))
    for row in rows:
        if not row['passed']:
            print('FAILED', row['name'], json.dumps({key: val for key, val in row['checks'].items() if not val}), file=sys.stderr)
    return 0 if passed == len(rows) else 1


if __name__ == '__main__':
    raise SystemExit(main())
