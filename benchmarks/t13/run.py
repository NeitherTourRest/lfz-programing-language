"""T13 frozen single-candidate evaluation, with both old and T12 baselines.

No interpreter source/profile mutation. Use --preflight for hashes only.
All paths and scratch data remain inside this project.
"""
import argparse
import ctypes
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import statistics
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'benchmarks/t12'))
from process_measure import capture


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def local(path):
    path = path.resolve()
    if not path.is_relative_to(ROOT):
        raise ValueError('path must remain inside project: ' + str(path))
    return path


def rel(path):
    return local(path).relative_to(ROOT).as_posix()


def source_hashes():
    return {rel(p): sha(p) for p in sorted((ROOT / 'src').glob('*.rs')) + [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock']}


def current_affinity():
    if os.name != 'nt':
        return None
    api = ctypes.WinDLL('kernel32', use_last_error=True)
    api.GetCurrentProcess.restype = ctypes.c_void_p
    api.GetProcessAffinityMask.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_size_t), ctypes.POINTER(ctypes.c_size_t)]
    process, system = ctypes.c_size_t(), ctypes.c_size_t()
    if not api.GetProcessAffinityMask(api.GetCurrentProcess(), ctypes.byref(process), ctypes.byref(system)):
        raise ctypes.WinError(ctypes.get_last_error())
    return {'process_mask': process.value, 'system_mask': system.value, 'modified': False}


def summary(samples):
    formal = [sample for sample in samples if sample['phase'] == 'formal']
    assert len(formal) == 7
    values = [sample['elapsed_ms'] for sample in formal]
    return {
        'median_ms': statistics.median(values), 'min_ms': min(values), 'max_ms': max(values),
        'median_user_ms': statistics.median(sample['user_ms'] for sample in formal),
        'median_kernel_ms': statistics.median(sample['kernel_ms'] for sample in formal),
        'runs': formal, 'warmup_runs': [sample for sample in samples if sample['phase'] == 'warmup'],
    }


def capabilities(temp):
    cases = []
    for n in (2000, 4000, 8000):
        value = {f'k{i}': i for i in range(n)}
        data = json.dumps(value, ensure_ascii=False, separators=(',', ':'), sort_keys=True).encode('utf-8')
        data_path = temp / f'json-{n}.json'
        data_path.write_bytes(data)
        literal = json.dumps(rel(data_path))
        for mode in ('parse', 'stringify'):
            source = f'#42\nlet text = readFile({literal})\n'
            if mode == 'stringify':
                source += 'let value = jsonParse(text)\n'
            op = f'let value = jsonParse(text)\n total += value["k{n-1}"]' if mode == 'parse' else 'total += len(jsonStringify(value))'
            source += 'var total = 0\nvar j = 0\nwhile j < 20 {\n ' + op + '\n j += 1\n}\nprint("RESULT total=${total}")\n'
            path = temp / f'json-{mode}-{n}.lfz'
            path.write_text(source, encoding='utf-8', newline='\n')
            cases.append({'name': 'json_' + mode, 'n': n, 'arguments': ['run', str(path)], 'stdin': '', 'expected_stdout': f'RESULT total={20 * (n-1 if mode == "parse" else len(data))}\n', 'source': source, 'source_sha256': sha(path), 'input_bytes': len(data), 'input_sha256': hashlib.sha256(data).hexdigest(), 'iterations': 20})
    source = 'print("RESULT ok=1")\n'
    cases.append({'name': 'cli_eval', 'n': 0, 'arguments': ['-e', source], 'stdin': '', 'expected_stdout': 'RESULT ok=1\n', 'source': source})
    cases.append({'name': 'cli_stdin', 'n': 0, 'arguments': ['-'], 'stdin': source, 'expected_stdout': 'RESULT ok=1\n', 'source': source})
    source = 'print(jsonStringify(argv()))\n'
    args = ['literal', '中文', '--json', '-e', '']
    cases.append({'name': 'cli_argv', 'n': 0, 'arguments': ['-e', source, '--'] + args, 'stdin': '', 'expected_stdout': json.dumps(args, ensure_ascii=False, separators=(',', ':')) + '\n', 'source': source})
    return cases


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--new-binary', type=Path, required=True)
    parser.add_argument('--expected-new-sha', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--preflight', action='store_true')
    args = parser.parse_args()
    protocol_path = ROOT / 'benchmarks/t13/protocol.json'
    protocol = json.loads(protocol_path.read_text(encoding='utf-8'))
    assert protocol['protocol'] == 'T13-single-candidate-dual-baseline-v1'
    for name, expected in {**protocol['original_baseline_files'], **protocol['supporting_scripts']}.items():
        assert sha(ROOT / name) == expected, ('frozen input changed', name)
    assert len(protocol['cases']) == 19
    assert (protocol['warmup_runs_per_label_case'], protocol['formal_runs_per_label_case'], protocol['settle_after_copy_seconds']) == (2, 7, 5)
    new = local(args.new_binary)
    out = local(args.output)
    if out.exists():
        raise FileExistsError('new output required; existing evidence cannot be overwritten')
    temp = ROOT / 'Temp/t13-perf'
    subject = temp / 'subject.exe'
    assert new != subject
    binaries = {label: local(ROOT / meta['path']) for label, meta in protocol['baselines'].items()}
    binaries['new'] = new
    expected_hashes = {label: meta['sha256'] for label, meta in protocol['baselines'].items()}
    expected_hashes['new'] = args.expected_new_sha.lower()
    assert len(expected_hashes['new']) == 64 and all(c in '0123456789abcdef' for c in expected_hashes['new'])
    actual_hashes = {label: sha(path) for label, path in binaries.items()}
    assert actual_hashes == expected_hashes
    assert expected_hashes['new'] not in (expected_hashes['old'], expected_hashes['t12']), 'candidate must contain changed bytes'
    start_sources = source_hashes()
    preflight = {'protocol_sha256': sha(protocol_path), 'hashes': actual_hashes, 'source_hashes': start_sources, 'affinity': current_affinity(), 'new_bytes': new.stat().st_size, 'output': rel(out), 'timing_performed': False}
    if args.preflight:
        print(json.dumps(preflight, ensure_ascii=False, indent=2))
        return
    temp.mkdir(parents=True, exist_ok=True)
    out.parent.mkdir(parents=True, exist_ok=True)
    # Reserve this exact evidence path before any launch. On error it remains a partial run.
    with out.open('x', encoding='utf-8') as handle:
        handle.write('{}\n')
    env = os.environ.copy()
    env.update(TEMP=str(temp), TMP=str(temp), PYTHONDONTWRITEBYTECODE='1')
    result = {'protocol': protocol, **preflight, 'timing_performed': True, 'status': 'running', 'timestamp_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'client_date': '2026-10-27', 'environment': {'os': platform.platform(), 'cpu': platform.processor(), 'logical_cpus': os.cpu_count(), 'python': sys.version, 'python_exe': sys.executable}, 'binary_sources': {label: {'path': rel(path), 'sha256': expected_hashes[label], 'bytes': path.stat().st_size} for label, path in binaries.items()}, 'cases': [], 'capabilities': []}

    def save():
        out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

    def measure(label, arguments, stdin, expected, entry):
        path = binaries[label]
        assert sha(path) == expected_hashes[label], ('binary source changed', label)
        shutil.copyfile(path, subject)
        assert sha(subject) == expected_hashes[label]
        time.sleep(protocol['settle_after_copy_seconds'])
        samples = []
        entry['in_progress_label'] = label
        entry['in_progress_samples'] = samples
        save()
        for i in range(9):
            assert sha(subject) == expected_hashes[label], ('subject changed', label)
            command = [str(subject)] + arguments
            process, elapsed_ms, cpu = capture(command, cwd=ROOT, env=env, input=stdin.encode('utf-8'), timeout=15)
            actual = process.stdout.decode('utf-8').replace('\r\n', '\n')
            sample = {'phase': 'warmup' if i < 2 else 'formal', 'index': i, 'sha256': expected_hashes[label], 'elapsed_ms': elapsed_ms, **cpu, 'exit': process.returncode, 'stdout': actual, 'stderr': process.stderr.decode('utf-8', 'replace')}
            samples.append(sample)
            save()
            assert process.returncode == 0 and actual == expected, ('oracle mismatch', label, entry['name'], entry['n'], sample)
        values = summary(samples)
        entry.pop('in_progress_label', None)
        entry.pop('in_progress_samples', None)
        return values

    try:
        save()
        labels = protocol['label_order']
        for index, row in enumerate(protocol['cases']):
            path = temp / f"{row['name']}-{row['n']}.lfz"
            path.write_text(row['source'], encoding='utf-8', newline='\n')
            assert sha(path) == row['source_sha256']
            entry = {**row, 'input_script': rel(path), 'label_order': labels[index % 3:] + labels[:index % 3], 'groups': {}}
            result['cases'].append(entry)
            for label in entry['label_order']:
                entry['groups'][label] = measure(label, ['run', str(path)], row['stdin'], row['expected_stdout'], entry)
                save()
            groups = entry['groups']
            limit = row['original_fixed_limit_ms']
            entry['absolute_status'] = 'PASS' if groups['new']['median_ms'] <= limit else ('ENVIRONMENT_INDETERMINATE' if groups['old']['median_ms'] > limit else 'FAIL')
            entry['new_over_old'] = groups['new']['median_ms'] / groups['old']['median_ms']
            entry['new_over_t12'] = groups['new']['median_ms'] / groups['t12']['median_ms']
            save()
            print(row['name'], row['n'], 'old/t12/new', *(round(groups[label]['median_ms'], 3) for label in labels), entry['absolute_status'], flush=True)
        for index, row in enumerate(capabilities(temp)):
            entry = {**row, 'label_order': ['t12', 'new'] if index % 2 == 0 else ['new', 't12'], 'groups': {}}
            result['capabilities'].append(entry)
            for label in entry['label_order']:
                entry['groups'][label] = measure(label, row['arguments'], row['stdin'], row['expected_stdout'], entry)
                save()
            entry['new_over_t12'] = entry['groups']['new']['median_ms'] / entry['groups']['t12']['median_ms']
            save()
            print(row['name'], row['n'], 't12/new', round(entry['groups']['t12']['median_ms'], 3), round(entry['groups']['new']['median_ms'], 3), flush=True)
        probe = next(row for row in protocol['cases'] if row['name'] == 'numeric_loop' and row['n'] == 500000)
        path = temp / 'numeric_loop-500000.lfz'
        # Probe the actual delivery path separately; do not replace any common-path result.
        samples = []
        result['dist_path_probe'] = {'name': 'dist_numeric_loop', 'n': 500000, 'path': rel(new), 'sha256': expected_hashes['new'], 'samples': samples}
        for i in range(9):
            assert sha(new) == expected_hashes['new']
            process, elapsed_ms, cpu = capture([str(new), 'run', str(path)], cwd=ROOT, env=env, input=probe['stdin'].encode('utf-8'), timeout=15)
            actual = process.stdout.decode('utf-8').replace('\r\n', '\n')
            samples.append({'phase': 'warmup' if i < 2 else 'formal', 'index': i, 'sha256': expected_hashes['new'], 'elapsed_ms': elapsed_ms, **cpu, 'exit': process.returncode, 'stdout': actual, 'stderr': process.stderr.decode('utf-8', 'replace')})
            save()
            assert process.returncode == 0 and actual == probe['expected_stdout']
        result['dist_path_probe'].update(summary(samples))
        end_sources = source_hashes()
        assert end_sources == start_sources, 'source/Cargo changed during window'
        assert sha(protocol_path) == result['protocol_sha256'], 'protocol changed during window'
        result['end_source_hashes'] = end_sources
        counts = {status: sum(row['absolute_status'] == status for row in result['cases']) for status in ('PASS', 'ENVIRONMENT_INDETERMINATE', 'FAIL')}
        struct = next(row for row in result['cases'] if row['name'] == 'struct_ops' and row['n'] == 8000)
        struct_target = struct['groups']['new']['median_ms'] <= struct['original_before_ms'] * .7
        result['assessment'] = {'absolute_counts': counts, 'struct8_original_30pct_target_pass': struct_target, 'all_19_absolute_pass': counts['PASS'] == 19, 'overall_original_efficiency_gate_pass': counts['PASS'] == 19 and struct_target, 'R304': 'MEASUREMENT_GATE_PASS_REQUIRES_INDEPENDENT_REVIEW' if counts['PASS'] == 19 and struct_target else 'OPEN', 'peak_rss_measured': False, 'all_outputs_correct': True}
        result['status'] = 'completed'
        result['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        save()
        print(json.dumps(result['assessment'], ensure_ascii=False), flush=True)
    except BaseException as error:
        result['status'] = 'aborted'
        result['error'] = repr(error)
        result['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        save()
        raise


if __name__ == '__main__':
    main()