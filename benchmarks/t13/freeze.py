"""Freeze the T13 measurement protocol before a new candidate is timed."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'benchmarks/t13/protocol.json'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    files = [ROOT / 'benchmarks/t12/results' / name for name in ('before.json', 'before-small.json')]
    rows = [row for path in files for row in json.loads(path.read_text(encoding='utf-8'))['cases']]
    assert len(rows) == 19
    protocol = {
        'protocol': 'T13-single-candidate-dual-baseline-v1',
        'frozen_on_client_date': '2026-10-27',
        'status': 'frozen before candidate timing; no timing performed during preparation',
        'whole_process': True,
        'clock': 'perf_counter_ns',
        'warmup_runs_per_label_case': 2,
        'formal_runs_per_label_case': 7,
        'timeout_seconds': 15,
        'settle_after_copy_seconds': 5,
        'affinity': 'default; no pinned CPU; inherited mask recorded',
        'execution_path': 'Temp/t13-perf/subject.exe',
        'case_order': 'original 19; label blocks rotate old/t12/new by case index mod 3',
        'label_order': ['old', 't12', 'new'],
        'new_capabilities': 'same 9 T12 JSON/CLI cases; t12/new order alternates by case; 20 JSON operations per process',
        'anomaly_policy': 'one complete window for one changed candidate; retain every value; fail/timeout/hash/oracle mismatch aborts with partial output; no automatic retry and no replacement of slow samples',
        'decision': 'new <= original fixed line => PASS; otherwise contemporaneous old also over => ENVIRONMENT_INDETERMINATE; otherwise FAIL; original struct8 >=30% reduction goal remains; both paired ratios always reported',
        'limitations': 'cannot guarantee background OS quiet; total includes startup/parse/IO/eval; CPU can quantize to zero; no peak RSS; JSON paired ratios descriptive and cannot establish pure throughput',
        'baselines': {
            'old': {'path': 'Temp/t12-perf/lfz-before.exe', 'sha256': '149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab'},
            't12': {'path': 'Temp/t13-perf/lfz-t12.exe', 'sha256': '47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f'},
        },
        'original_baseline_files': {path.relative_to(ROOT).as_posix(): sha(path) for path in files},
        'supporting_scripts': {name: sha(ROOT / name) for name in ('benchmarks/t12/process_measure.py', 'benchmarks/t13/run.py')},
        'cases': [dict(name=row['name'], n=row['n'], source=row['source'], source_sha256=hashlib.sha256(row['source'].encode('utf-8')).hexdigest(), stdin=row['stdin'], expected_stdout=row['expected_stdout'], original_before_ms=row['median_ms'], original_fixed_limit_ms=max(row['median_ms'] * (1.3 if row['name'] == 'noop' else 1.2), row['median_ms'] + 5)) for row in rows],
    }
    for label, meta in protocol['baselines'].items():
        assert sha(ROOT / meta['path']) == meta['sha256'], label
    with OUT.open('x', encoding='utf-8', newline='\n') as handle:
        json.dump(protocol, handle, ensure_ascii=False, indent=2)
        handle.write('\n')
    print(json.dumps({'protocol_sha256': sha(OUT), 'cases': len(rows), 'timing_performed': False}))

if __name__ == '__main__':
    main()