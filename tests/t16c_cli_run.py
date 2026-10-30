"""T16-C public process contract tests; never imports product code or compiles it."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--binary', type=Path, required=True)
    ap.add_argument('--expected-sha', required=True)
    ap.add_argument('--output', type=Path, required=True)
    ap.add_argument('--prepare-only', action='store_true')
    a = ap.parse_args()
    exe, out = a.binary.resolve(), a.output.resolve()
    assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists()
    w = ROOT / 'Temp/T16-C-test'
    w.mkdir(parents=True, exist_ok=True)
    out.parent.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env.update(TEMP=str(w), TMP=str(w), PYTHONDONTWRITEBYTECODE='1')
    rows, cases = [], []

    def file(name, body):
        p = w / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text('#42\n' + body + '\n', encoding='utf8', newline='\n')
        return p

    def add(name, files, body, *, mode='run', json_mode=False, code=0,
            stdout='', stderr='', error=None, error_file=None, line=None,
            frame_files=None, args=(), stdin=None, watch=(), contains=()):
        home = name
        for path, src in files.items():
            file(home + '/' + path, src)
        entry = file(home + '/main.lfz', body)
        command = [mode] + (['--json'] if json_mode else [])
        if stdin == 'eval':
            command += ['-e', body]
            data = b''
        elif stdin == 'stdin':
            command += ['-']
            data = body.encode('utf8')
        else:
            command += [str(entry)]
            data = b''
        if args:
            command += ['--', *args]
        cases.append(dict(name=name, args=command, data=data, cwd=entry.parent,
                          mode=mode, json_mode=json_mode, code=code,
                          stdout=stdout, stderr=stderr, error=error,
                          error_file=error_file, line=line, frame_files=frame_files,
                          watch=watch, contains=contains))

    # Serial dependency postorder, canonical diamond aliases, single initialization.
    diamond = {'leaf.lfz': 'print("leaf")\nvar n=1\nfn get()=>n\nexport n,get',
               'left.lfz': 'import d from "./leaf.lfz"\nprint("left")\nexport d',
               'right.lfz': 'import d from "./sub/../leaf.lfz"\nprint("right")\nexport d',
               'sub/empty.lfz': ''}
    body = 'import a from "./left.lfz"\nimport b from "./right.lfz"\nassert(a.d.n==1)\na.d.n=8\nassert(b.d.n==8)\nassert(a.d.get()==1)\nprint("entry")'
    add('diamond_once', diamond, body, stdout='leaf\nleft\nright\nentry\n')
    add('fresh_process', diamond, body, stdout='leaf\nleft\nright\nentry\n')
    add('independent_globals', {'lib.lfz': 'let n=7\nfn get()=>n\nexport get'},
        'import m from "./lib.lfz"\nlet n=99\nassert(m.get()==7)\nassert(n==99)')
    add('home_closure', {'lib.lfz': 'let n=7\nfn make() {\nvar x=2\n()=>{\nx+=1\nn+x\n}\n}\nexport make'},
        'import m from "./lib.lfz"\nlet n=99\nlet f=m.make()\nassert(f()==10)\nassert(f()==11)')
    add('namespace_snapshot_shared', {'lib.lfz': 'var n=1\nlet a=[1]\nlet s={x:2}\nn=3\nfn get()=>n\nfn read()=>a[0]+s.x\nexport n,a,s,get,read'},
        'import m from "./lib.lfz"\nassert(m.n==3)\nm.n=8\nassert(m.get()==3)\nm.a[0]=9\nm.s.x=10\nassert(m.read()==19)\nassert(keys(m)==["a","n","s"])')
    add('self_methods', {'lib.lfz': 'let base=4\nstruct Box {x:2,fn get()=>base+self.x}\nexport Box'},
        'import m from "./lib.lfz"\nlet base=99\nlet B=m.Box\nlet b=B{x:3}\nassert(b.get()==7)\nassert(b["get"]()==7)')
    add('argv_snapshot', {'lib.lfz': 'let init=argv()\nfn get()=>argv()\nexport init,get'},
        'import m from "./lib.lfz"\nassert(m.init==["","雪","--json"])\nlet a=m.get()\na[0]="changed"\nassert(argv()==["","雪","--json"])\nassert(m.get()==m.init)', args=('', '雪', '--json'))
    add('empty_namespace', {'lib.lfz': 'let hidden=5'},
        'import m from "./lib.lfz"\nassert(type(m)=="struct")\nassert(keys(m)==[])')
    add('template_definition_globals', {'lib.lfz': 'let seed=7\nstruct Point {x:seed,a:[1]}\nexport Point'},
        'import m from "./lib.lfz"\nlet seed=99\nlet P=m.Point\nlet p=P{}\nlet q=P{}\nassert(p.x==7)\np.a[0]=8\nassert(q.a[0]==1)\nlet r=P{x:seed}\nassert(r.x==99)')
    add('template_local_cells', {'lib.lfz': 'fn make(){\nvar n=4\nstruct T{x:n}\nn=6\nT\n}\nexport make'},
        'import m from "./lib.lfz"\nlet T=m.make()\nlet n=99\nassert(T{}.x==6)')
    add('template_self', {'lib.lfz': 'struct Factory {seed:5,fn make(){\nstruct T{x:self.seed}\nT\n}}\nexport Factory'},
        'import m from "./lib.lfz"\nlet F=m.Factory\nlet f=F{seed:8}\nlet T=f.make()\nassert(T{}.x==8)')
    add('template_default_order', {'lib.lfz': 'var n=0\nfn tick(){\nn+=1\nn\n}\nstruct T{a:tick(),b:tick()}\nfn count()=>n\nexport T,count'},
        'import m from "./lib.lfz"\nlet T=m.T\nlet t=T{a:90}\nassert(t.a==90)\nassert(t.b==2)\nassert(m.count()==2)')
    failing = {'lib.lfz': 'fn bad(){\n1/0\n}\nexport bad'}
    add('cross_error_json', failing, 'import m from "./lib.lfz"\nm.bad()',
        json_mode=True, code=2, error='ZeroDivisionError', error_file='lib.lfz', line=3,
        frame_files=['main.lfz', 'lib.lfz'])
    add('cross_error_human', failing, 'import m from "./lib.lfz"\nm.bad()',
        code=2, contains=('lib.lfz', '1/0', 'ZeroDivisionError'))
    add('attempt_home_restore', failing,
        'import m from "./lib.lfz"\nlet n=99\nlet r=attempt(()=>m.bad())\nassert(!r.ok)\nassert(n==99)\nassert(attempt(()=>8).value==8)')
    add('attempt_new_error_source', failing,
        'import m from "./lib.lfz"\nlet r=attempt(()=>m.bad())\n1/0',
        json_mode=True, code=2, error='ZeroDivisionError', error_file='main.lfz', line=4,
        frame_files=['main.lfz'])
    add('caller_arity_source', failing, 'import m from "./lib.lfz"\nm.bad(1)',
        json_mode=True, code=2, error='TypeError', error_file='main.lfz', line=3,
        frame_files=['main.lfz'])
    add('default_error_source', {'lib.lfz': 'struct Bad{x:1/0}\nexport Bad'},
        'import m from "./lib.lfz"\nlet B=m.Bad\nB{}', json_mode=True,
        code=2, error='ZeroDivisionError', error_file='lib.lfz', line=2,
        frame_files=['main.lfz', 'lib.lfz'])
    add('override_error_source', {'lib.lfz': 'struct T{x:3}\nexport T'},
        'import m from "./lib.lfz"\nlet T=m.T\nT{x:1/0}', json_mode=True,
        code=2, error='ZeroDivisionError', error_file='main.lfz', line=4,
        frame_files=['main.lfz'])
    # Invalid frontend graphs are observable via CLI; forged host graph invariants belong to unit tests.
    for name, badsrc in [('bad_graph_syntax', 'let x ='),
                         ('bad_graph_export', 'export missing'),
                         ('bad_graph_cycle', 'import m from "./main.lfz"')]:
        marker = w / name / 'sentinel.txt'
        marker.parent.mkdir(parents=True, exist_ok=True)
        marker.write_bytes(b'ORIGINAL')
        add(name, {'early.lfz': 'writeFile("sentinel.txt","BAD")\nprint("BAD")', 'bad.lfz': badsrc},
            'import a from "./early.lfz"\nimport b from "./bad.lfz"\nprint("ENTRY")',
            json_mode=True, code=2, error='SyntaxError', watch=(marker,))
    marker = w / 'init_fail_stops' / 'sentinel.txt'
    marker.parent.mkdir(parents=True, exist_ok=True)
    marker.write_bytes(b'ORIGINAL')
    add('init_fail_stops', {'bad.lfz': '1/0', 'later.lfz': 'writeFile("sentinel.txt","BAD")'},
        'import b from "./bad.lfz"\nimport l from "./later.lfz"\nwriteFile("sentinel.txt","ENTRY")',
        json_mode=True, code=2, error='ZeroDivisionError', error_file='bad.lfz', line=2,
        frame_files=['bad.lfz'], watch=(marker,))
    for flavor, effect in [('io', 'writeFile("sentinel.txt","BAD")'),
                           ('input', 'input("BAD")'), ('loop', 'while true {}')]:
        name = 'check_no_init_' + flavor
        marker = w / name / 'sentinel.txt'
        marker.parent.mkdir(parents=True, exist_ok=True)
        marker.write_bytes(b'ORIGINAL')
        add(name, {'lib.lfz': 'print("BAD")\n' + effect},
            'import m from "./lib.lfz"', mode='check', json_mode=True, watch=(marker,))
    for source in ('eval', 'stdin'):
        add('source_' + source, {'lib.lfz': 'let n=7\nexport n'},
            'import m from "./lib.lfz"\nassert(m.n==7)', stdin=source)
    add('json_program_output', {'lib.lfz': 'print("DEP")'},
        'import m from "./lib.lfz"\nprint("ROOT")', json_mode=True, stderr='DEP\nROOT\n')

    if a.prepare_only:
        result = {'schema': 'LFZ-T16-C-PREPARED-1', 'cases_total': len(cases),
                  'case_names': [c['name'] for c in cases], 'executed': False}
    else:
        digest = sha(exe)
        assert digest == a.expected_sha.lower(), 'binary fingerprint mismatch'
        for c in cases:
            assert sha(exe) == digest, 'binary changed during acceptance'
            before = {str(p): p.read_bytes().hex() if p.exists() else None for p in c['watch']}
            checks = {}
            row = {'name': c['name'], 'args': c['args'], 'cwd': str(c['cwd']),
                   'stdin_hex': c['data'].hex(), 'before': before}
            try:
                r = subprocess.run([str(exe), *c['args']], cwd=c['cwd'], env=env,
                                   input=c['data'], capture_output=True, timeout=8)
                stdout, stderr = r.stdout.decode('utf8'), r.stderr.decode('utf8')
                row.update(exit=r.returncode, stdout=stdout, stderr=stderr)
                checks['exit'] = r.returncode == c['code']
                if c['json_mode']:
                    obj = json.loads(stdout)
                    base = ['ok', 'mode', 'executed'] if c['mode'] == 'check' else ['ok']
                    keys = base + (['error', 'message', 'file', 'line', 'col', 'traceback'] if c['code'] else [])
                    checks['keys'] = list(obj) == keys
                    checks['ok'] = obj.get('ok') == (c['code'] == 0)
                    checks['one_line'] = stdout.count('\n') == 1
                    checks['stderr'] = stderr == c['stderr']
                    if c['mode'] == 'check':
                        checks['check_flags'] = obj.get('mode') == 'check' and obj.get('executed') is False
                    if c['error']:
                        checks['error'] = obj.get('error') == c['error']
                    if c['error_file']:
                        checks['file'] = Path(obj.get('file', '')).name == c['error_file']
                    if c['line'] is not None:
                        checks['line'] = obj.get('line') == c['line']
                    if c['frame_files'] is not None:
                        checks['frame_files'] = [Path(f.get('file', '')).name for f in obj.get('traceback', [])] == c['frame_files']
                else:
                    checks['stdout'] = stdout == c['stdout']
                    if c['code'] == 0:
                        checks['stderr'] = stderr == c['stderr']
                    for text in c['contains']:
                        checks['contains_' + text] = text in stderr
                after = {str(p): p.read_bytes().hex() if p.exists() else None for p in c['watch']}
                checks['sentinels'] = before == after
                row['after'] = after
            except (ValueError, UnicodeError, subprocess.TimeoutExpired) as e:
                row['exception'] = repr(e)
                checks['completed'] = False
            row.update(checks=checks, passed=all(checks.values()))
            rows.append(row)
        assert sha(exe) == digest
        result = {'schema': 'LFZ-T16-C-CLI-1', 'binary': str(exe.relative_to(ROOT)),
                  'binary_sha256': digest, 'bytes': exe.stat().st_size,
                  'checks_total': len(rows), 'checks_passed': sum(r['passed'] for r in rows),
                  'script_sha256': sha(Path(__file__)), 'rows': rows}
    result['timestamp_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf8', newline='\n')
    print(json.dumps({k: v for k, v in result.items() if k not in ('rows', 'case_names')}, ensure_ascii=False))
    return 0 if a.prepare_only or result['checks_passed'] == result['checks_total'] else 1


if __name__ == '__main__':
    sys.exit(main())
