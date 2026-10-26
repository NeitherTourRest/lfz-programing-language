"""sortviz参数与默认兼容自检；旧版本exe必须另行提供。"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lfz', default='dist/lfz.exe')
    parser.add_argument('--before-exe', required=True)
    parser.add_argument('--output', default='Temp/T12/app-sortviz-check.json')
    args = parser.parse_args()
    exe, before = [(ROOT / x).resolve() for x in (args.lfz, args.before_exe)]
    report = (ROOT / args.output).resolve()
    if not report.is_relative_to(ROOT) or report.exists():
        parser.error('output must be a new project path')
    temp = ROOT / 'Temp/T12'
    temp.mkdir(parents=True, exist_ok=True)
    checks = []
    with tempfile.TemporaryDirectory(prefix='sortviz-', dir=temp) as folder:
        work = Path(folder)
        env = dict(os.environ, TEMP=str(work), TMP=str(work))
        original = subprocess.run(['git', 'show', 'v1.3.0:app/sortviz.lfz'], cwd=ROOT, capture_output=True, check=True).stdout
        old_source = work / 'before.lfz'
        old_source.write_bytes(original)
        old = subprocess.run([str(before), 'run', str(old_source)], cwd=ROOT, env=env, capture_output=True, timeout=60)
        new = subprocess.run([str(exe), 'run', 'app/sortviz.lfz'], cwd=ROOT, env=env, capture_output=True, timeout=60)
        assert old.returncode == new.returncode == 0 and not old.stderr and not new.stderr
        assert old.stdout == new.stdout, 'default stdout differs'
        checks.append({'name': 'default_byte_identical_to_v1.3.0', 'pass': True,
                       'stdout_sha256': hashlib.sha256(new.stdout).hexdigest(), 'stdout': new.stdout.decode('utf-8')})
        flags = ['--small', '4', '--big', '30', '--seed', '7', '--no-frames']
        cmd = [str(exe), 'run', 'app/sortviz.lfz', '--'] + flags
        custom = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, timeout=30)
        repeated = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, timeout=30)
        text = custom.stdout.decode('utf-8')
        assert custom.returncode == repeated.returncode == 0 and not custom.stderr and not repeated.stderr
        assert custom.stdout == repeated.stdout and 'N_SMALL=4  N_BIG=30  SEED=7' in text
        assert 'SHOW_FRAMES=false' in text and 'bubble t=0' not in text
        assert '自测 150 次排序，失败 0 次' in text
        checks.append({'name': 'parameters_reproducible_150_selfchecks', 'pass': True, 'command': cmd, 'stdout': text})
        machine = subprocess.run([str(exe), '--json', 'run', 'app/sortviz.lfz', '--'] + flags,
                                 cwd=ROOT, env=env, capture_output=True, timeout=30)
        assert machine.returncode == 0 and machine.stdout == b'{"ok":true}\n' and machine.stderr == custom.stdout
        checks.append({'name': 'machine_channels', 'pass': True, 'stdout': machine.stdout.decode('utf-8')})
        bad = [[], ['--small'], ['--small', '0'], ['--small', '101'], ['--big', '0'],
               ['--big', '2001'], ['--seed', 'bad'], ['--unknown'], ['--no-frames', '--no-frames']]
        # 空列表是合法默认，已做逐字比较；下面只跑错误参数。
        for flags in bad[1:]:
            result = subprocess.run([str(exe), 'run', 'app/sortviz.lfz', '--'] + flags,
                                    cwd=ROOT, env=env, capture_output=True, timeout=15)
            assert result.returncode == 2 and not result.stdout, flags
            checks.append({'name': 'reject_' + '_'.join(flags), 'pass': True, 'exit': result.returncode,
                           'stderr': result.stderr.decode('utf-8')})
    payload = {'schema': 'LFZ-T12-sortviz-check-v1', 'date': '2026-10-25',
               'lfz_sha256': hashlib.sha256(exe.read_bytes()).hexdigest(),
               'before_lfz_sha256': hashlib.sha256(before.read_bytes()).hexdigest(),
               'source_sha256': hashlib.sha256((ROOT / 'app/sortviz.lfz').read_bytes()).hexdigest(),
               'passed': len(checks), 'checks': checks}
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'passed': len(checks), 'report': str(report)}, ensure_ascii=False))


if __name__ == '__main__':
    main()
