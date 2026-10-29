"""Recompute T13 raw evidence without launching the interpreter or timing."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

ROOT = Path(__file__).resolve().parents[2]
RAW = ROOT / 'benchmarks/t13/results/final-window.json'
OUT = ROOT / 'benchmarks/t13/results/assessment.json'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, default=OUT)
    args = parser.parse_args()
    out = args.output.resolve()
    assert out.is_relative_to(ROOT) and not out.exists()
    out.parent.mkdir(parents=True, exist_ok=True)
    raw = json.loads(RAW.read_text(encoding='utf-8'))
    protocol_path = ROOT / 'benchmarks/t13/protocol.json'
    protocol = json.loads(protocol_path.read_text(encoding='utf-8'))
    assert raw['status'] == 'completed' and raw['protocol'] == protocol
    assert raw['protocol_sha256'] == sha(protocol_path)
    assert raw['source_hashes'] == raw['end_source_hashes']
    assert len(raw['cases']) == 19 and len(raw['capabilities']) == 9
    counts = {'PASS': 0, 'ENVIRONMENT_INDETERMINATE': 0, 'FAIL': 0}
    formal_count = warmup_count = 0
    table = []

    def check_group(group, label, expected):
        nonlocal formal_count, warmup_count
        runs = group['runs']
        warmups = group['warmup_runs']
        assert len(runs) == 7 and len(warmups) == 2
        for phase, samples, expected_indices in [('formal', runs, list(range(2, 9))), ('warmup', warmups, [0, 1])]:
            assert [sample['index'] for sample in samples] == expected_indices
            for sample in samples:
                assert sample['phase'] == phase
                assert sample['sha256'] == raw['hashes'][label]
                assert sample['exit'] == 0 and sample['stdout'] == expected
                assert sample['elapsed_ms'] >= 0 and sample['user_ms'] >= 0 and sample['kernel_ms'] >= 0
        formal_count += len(runs)
        warmup_count += len(warmups)
        values = [sample['elapsed_ms'] for sample in runs]
        assert group['median_ms'] == statistics.median(values)
        assert group['min_ms'] == min(values) and group['max_ms'] == max(values)
        assert group['median_user_ms'] == statistics.median(sample['user_ms'] for sample in runs)
        assert group['median_kernel_ms'] == statistics.median(sample['kernel_ms'] for sample in runs)

    for index, (row, frozen) in enumerate(zip(raw['cases'], protocol['cases'], strict=True)):
        for key, value in frozen.items():
            assert row[key] == value
        assert hashlib.sha256(row['source'].encode('utf-8')).hexdigest() == row['source_sha256']
        labels = protocol['label_order']
        assert row['label_order'] == labels[index % 3:] + labels[:index % 3]
        assert set(row['groups']) == {'old', 't12', 'new'}
        for label, group in row['groups'].items():
            check_group(group, label, row['expected_stdout'])
        medians = {label: row['groups'][label]['median_ms'] for label in ('old', 't12', 'new')}
        limit = max(row['original_before_ms'] * (1.3 if row['name'] == 'noop' else 1.2), row['original_before_ms'] + 5)
        assert limit == row['original_fixed_limit_ms']
        status = 'PASS' if medians['new'] <= limit else ('ENVIRONMENT_INDETERMINATE' if medians['old'] > limit else 'FAIL')
        assert row['absolute_status'] == status
        assert row['new_over_old'] == medians['new'] / medians['old']
        assert row['new_over_t12'] == medians['new'] / medians['t12']
        counts[status] += 1
        table.append({'name': row['name'], 'n': row['n'], **{label + '_wall_ms': ms for label, ms in medians.items()}, **{label + '_user_ms': row['groups'][label]['median_user_ms'] for label in ('old', 't12', 'new')}, **{label + '_kernel_ms': row['groups'][label]['median_kernel_ms'] for label in ('old', 't12', 'new')}, 'original_limit_ms': limit, 'new_over_old': row['new_over_old'], 'new_over_t12': row['new_over_t12'], 'absolute_status': status})
    for index, row in enumerate(raw['capabilities']):
        assert row['label_order'] == (['t12', 'new'] if index % 2 == 0 else ['new', 't12'])
        assert set(row['groups']) == {'t12', 'new'}
        for label, group in row['groups'].items():
            check_group(group, label, row['expected_stdout'])
        assert row['new_over_t12'] == row['groups']['new']['median_ms'] / row['groups']['t12']['median_ms']
    probe = raw['dist_path_probe']
    expected = next(row['expected_stdout'] for row in protocol['cases'] if row['name'] == 'numeric_loop' and row['n'] == 500000)
    check_group(probe, 'new', expected)
    assert probe['sha256'] == raw['hashes']['new']
    assert formal_count == 532 and warmup_count == 152
    assert counts == raw['assessment']['absolute_counts']
    struct = next(row for row in table if row['name'] == 'struct_ops' and row['n'] == 8000)
    struct_target = next(row['original_before_ms'] for row in protocol['cases'] if row['name'] == 'struct_ops' and row['n'] == 8000) * .7
    assert raw['assessment']['struct8_original_30pct_target_pass'] == (struct['new_wall_ms'] <= struct_target)
    result = {'status': 'RECOMPUTATION_PASS', 'interpreter_launches': 0, 'raw_sha256': sha(RAW), 'protocol_sha256': sha(protocol_path), 'hashes': raw['hashes'], 'absolute_counts': counts, 'R304': 'OPEN' if counts['PASS'] != 19 else 'REQUIRES_INDEPENDENT_REVIEW', 'formal_runs_checked': formal_count, 'warmup_runs_checked': warmup_count, 'all_outputs_correct': True, 'sources_unchanged_in_window': True, 'struct8_original_30pct_target_pass': struct['new_wall_ms'] <= struct_target, 'cases': table}
    with out.open('x', encoding='utf-8', newline='\n') as handle:
        json.dump(result, handle, ensure_ascii=False, indent=2)
        handle.write('\n')
    print(json.dumps({key: value for key, value in result.items() if key != 'cases'}, ensure_ascii=False))
    print('name n old t12 new new/old new/t12 limit status')
    for row in table:
        print(row['name'], row['n'], *[round(row[key], 4) for key in ('old_wall_ms', 't12_wall_ms', 'new_wall_ms', 'new_over_old', 'new_over_t12', 'original_limit_ms')], row['absolute_status'])
    print('capabilities')
    for row in raw['capabilities']:
        print(row['name'], row['n'], round(row['groups']['t12']['median_ms'], 4), round(row['groups']['new']['median_ms'], 4), round(row['new_over_t12'], 4))
    print('dist', probe['median_ms'])


if __name__ == '__main__':
    main()