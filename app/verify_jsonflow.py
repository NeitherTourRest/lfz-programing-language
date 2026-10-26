"""jsonflow应用自检；仅标准库，所有临时文件在项目Temp。"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / 'app/jsonflow.lfz'
SENTINEL = b'preserve-existing-output\n'


def encoded(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'), allow_nan=False) + '\n').encode('utf-8')


def equal(a, b):
    # LFZ bool不与int等同；int/float为精确数值比较，Python此处一致。
    if isinstance(a, bool) or isinstance(b, bool):
        return type(a) is type(b) and a == b
    return a == b


def oracle(config, records):
    condition = config['where']
    return [{field: record[field] for field in config['select']} for record in records
            if condition is None or equal(record[condition['field']], condition['equals'])]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lfz', default='dist/lfz.exe')
    parser.add_argument('--output', default='Temp/T12/app-jsonflow-check.json')
    args = parser.parse_args()
    exe = (ROOT / args.lfz).resolve()
    report = (ROOT / args.output).resolve()
    if not report.is_relative_to(ROOT) or report.exists():
        parser.error('output must be a new path inside project')
    temporary = ROOT / 'Temp/T12'
    temporary.mkdir(parents=True, exist_ok=True)
    checks = []
    base = {'where': {'field': 'status', 'equals': 'done'}, 'select': ['id', 'title']}
    records = [{'id': 1, 'title': '准备🙂', 'status': 'done'},
               {'id': 2, 'title': '工作', 'status': 'todo'},
               {'id': 9007199254740993, 'title': '${literal}', 'status': 'done'}]
    cases = [
        ('fixture', base, records, True, False),
        ('empty', base, [], True, False),
        ('no_match', base, records[1:2], True, False),
        ('select_all', {'where': None, 'select': ['title']}, records, True, False),
        ('unicode_key', {'where': {'field': '状态', 'equals': True}, 'select': ['名称🙂']},
         [{'状态': True, '名称🙂': '成功'}, {'状态': 1, '名称🙂': 'bool不是int'}], True, False),
        ('numeric_equality', {'where': {'field': 'n', 'equals': 1}, 'select': ['n']},
         [{'n': 1.0}, {'n': True}, {'n': 1}, {'n': 2}], True, False),
        ('null_equality', {'where': {'field': 'n', 'equals': None}, 'select': ['n']},
         [{'n': None}, {'n': False}], True, False),
        ('nested_projection', {'where': None, 'select': ['z', 'a']},
         [{'z': [None, True, '中文'], 'a': {'b': 1, 'a': 2}}], True, False),
        ('machine_channels', base, records, True, True),
        ('unknown_config', dict(base, extra=True), records, False, False),
        ('missing_config_field', {'select': ['id']}, records, False, False),
        ('config_scalar', 42, records, False, False),
        ('bad_select_type', dict(base, select='id'), records, False, False),
        ('empty_select', dict(base, select=[]), records, False, False),
        ('duplicate_select', dict(base, select=['id', 'id']), records, False, False),
        ('nonstring_select', dict(base, select=[1]), records, False, False),
        ('empty_select_field', dict(base, select=['']), records, False, False),
        ('where_scalar', dict(base, where=1), records, False, False),
        ('unknown_where', dict(base, where={'field': 'status', 'equals': 'done', 'extra': 1}), records, False, False),
        ('missing_equals', dict(base, where={'field': 'status'}), records, False, False),
        ('nonscalar_equals', dict(base, where={'field': 'status', 'equals': []}), records, False, False),
        ('bad_where_field', dict(base, where={'field': 1, 'equals': 'done'}), records, False, False),
        ('empty_where_field', dict(base, where={'field': '', 'equals': 'done'}), records, False, False),
        ('input_scalar', base, {}, False, False),
        ('record_scalar', base, [1], False, False),
        ('missing_selected', base, [{'id': 1, 'status': 'done'}], False, False),
        ('missing_where', base, [{'id': 1, 'title': 'x'}], False, False),
        ('bad_unmatched_record', base, [{'id': 1, 'status': 'todo'}], False, False),
    ]
    with tempfile.TemporaryDirectory(prefix='jsonflow-', dir=temporary) as td:
        work = Path(td)
        env = dict(os.environ, TEMP=str(work), TMP=str(work))
        for name, config, data, success, machine in cases:
            folder = work / name
            folder.mkdir()
            cfg, inp, out = [folder / file for file in ('config.json', 'input.json', 'output.json')]
            cfg.write_bytes(encoded(config))
            inp.write_bytes(encoded(data))
            out.write_bytes(SENTINEL)
            command = [str(exe), 'run'] + (['--json'] if machine else []) + [str(APP), '--', str(cfg), str(inp), str(out)]
            result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, timeout=15)
            assert cfg.read_bytes() == encoded(config) and inp.read_bytes() == encoded(data), name + ': inputs mutated'
            if success:
                expected = oracle(config, data)
                assert result.returncode == 0, (name, result.stderr.decode('utf-8'))
                assert out.read_bytes() == encoded(expected), (name, out.read_bytes(), encoded(expected))
                summary = {'input': len(data), 'output': len(expected), 'path': str(out)}
                if machine:
                    assert result.stdout == b'{"ok":true}\n', (name, result.stdout)
                    assert json.loads(result.stderr) == summary, (name, result.stderr)
                else:
                    assert json.loads(result.stdout) == summary and not result.stderr, (name, result.stdout, result.stderr)
            else:
                assert result.returncode == 2 and not result.stdout, (name, result.returncode, result.stdout)
                assert out.read_bytes() == SENTINEL, name + ': existing output truncated'
            checks.append({'name': name, 'pass': True, 'exit': result.returncode,
                           'stdout': result.stdout.decode('utf-8'), 'stderr': result.stderr.decode('utf-8'),
                           'output_sha256': hashlib.sha256(out.read_bytes()).hexdigest()})
        # 原始坏JSON、路径与参数错误同样不得提前写文件。
        cfg, inp, out = [work / x for x in ('config.json', 'input.json', 'output.json')]
        specials = [
            ('bad_config_json', b'{', encoded(records), [str(cfg), str(inp), str(out)]),
            ('duplicate_config_json', b'{"where":null,"where":null,"select":["id"]}', encoded(records), [str(cfg), str(inp), str(out)]),
            ('bad_input_json', encoded(base), b'[', [str(cfg), str(inp), str(out)]),
            ('missing_config_path', encoded(base), encoded(records), [str(work / 'missing.json'), str(inp), str(out)]),
            ('empty_path', encoded(base), encoded(records), ['', str(inp), str(out)]),
            ('missing_argument', encoded(base), encoded(records), [str(cfg), str(inp)]),
            ('extra_argument', encoded(base), encoded(records), [str(cfg), str(inp), str(out), 'extra']),
            ('output_is_input', encoded(base), encoded(records), [str(cfg), str(inp), str(inp)]),
            ('output_is_config', encoded(base), encoded(records), [str(cfg), str(inp), str(cfg)]),
        ]
        for name, cbytes, ibytes, argv in specials:
            cfg.write_bytes(cbytes)
            inp.write_bytes(ibytes)
            out.write_bytes(SENTINEL)
            command = [str(exe), 'run', str(APP), '--'] + argv
            result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, timeout=15)
            assert result.returncode == 2 and not result.stdout, (name, result.returncode)
            assert cfg.read_bytes() == cbytes and inp.read_bytes() == ibytes and out.read_bytes() == SENTINEL, name
            checks.append({'name': name, 'pass': True, 'exit': result.returncode,
                           'stdout': result.stdout.decode('utf-8'), 'stderr': result.stderr.decode('utf-8')})
    payload = {'schema': 'LFZ-T12-app-check-v1', 'date': '2026-10-25', 'lfz': str(exe),
               'lfz_sha256': hashlib.sha256(exe.read_bytes()).hexdigest(),
               'app_sha256': hashlib.sha256(APP.read_bytes()).hexdigest(), 'passed': len(checks), 'checks': checks}
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'passed': len(checks), 'report': str(report)}, ensure_ascii=False))


if __name__ == '__main__':
    main()
