"""主调度冻结后准备单题workspace。只复制学生允许看到的输入。"""
import argparse
from datetime import datetime, timedelta, timezone
import hashlib
import json
from pathlib import Path
import shutil

PACKAGE = Path(__file__).resolve().parent
ROOT = PACKAGE.parents[3]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--task', choices=('C01', 'C02', 'C03', 'C04'), required=True)
    parser.add_argument('--run-id', required=True)
    args = parser.parse_args()
    assert args.run_id.isascii() and args.run_id.replace('-', '').isalnum()
    manifest = json.loads(args.manifest.read_text(encoding='utf8'))
    assert manifest['status'] == 'FROZEN' and manifest['independent_review']
    for name, expected in manifest['files'].items():
        assert sha(PACKAGE / name) == expected, name
    candidate = manifest['final_binary']
    binary = ROOT / candidate['path']
    assert sha(binary) == candidate['sha256'] and binary.stat().st_size == candidate['bytes']
    docs = manifest['final_ai_docs']
    assert len(docs) == 5
    for name, expected in docs.items():
        assert sha(ROOT / name) == expected, name
    run = ROOT / 'Temp/agent-benchmark-v3' / args.run_id
    workspace = run / 'students' / args.task
    shutil.copytree(PACKAGE / 'student' / args.task, workspace)
    shutil.copyfile(binary, workspace / 'lfz.exe')
    shutil.copyfile(PACKAGE / 'student_cycle.py', workspace / 'student_cycle.py')
    for name in docs:
        target = 'AI-GUIDE.md' if name.startswith('docs/guide/ai/') else Path(name).name
        shutil.copyfile(ROOT / name, workspace / target)
    start = datetime.now(timezone.utc)
    config = dict(task=args.task, workspace=str(workspace), binary=str(workspace / 'lfz.exe'), binary_sha256=candidate['sha256'], start_utc=start.isoformat(), deadline_utc=(start + timedelta(seconds=1200)).isoformat())
    (workspace / 'cycle-config.json').write_text(json.dumps(config, indent=2) + '\n', encoding='utf8')
    receipt = run / 'dispatch-receipts' / (args.task + '.json')
    receipt.parent.mkdir(exist_ok=True)
    inputs = {str(path.relative_to(workspace)): sha(path) for path in workspace.rglob('*') if path.is_file()}
    receipt.write_text(json.dumps(dict(config=config, inputs=inputs, manifest_sha256=sha(args.manifest)), indent=2) + '\n', encoding='utf8')
    print(receipt)


if __name__ == '__main__':
    main()
