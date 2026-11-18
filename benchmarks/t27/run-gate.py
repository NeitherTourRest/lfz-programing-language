"""Run the frozen T13 protocol once against the final T27 candidate."""
import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUN = ROOT / "benchmarks/t13/run.py"



def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def sources():
    inputs = sorted((ROOT / "src").rglob("*.rs")) + [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in inputs}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--expected-sha", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--preflight", action="store_true")
    args = parser.parse_args()
    subject = args.binary.resolve()
    sha256 = args.expected_sha.lower()
    assert subject.is_relative_to(ROOT / "Temp")
    output = args.output.resolve()
    assert output.is_relative_to(ROOT / "Temp") and not output.exists()
    receipt = output.with_suffix(".receipt.json")
    assert not receipt.exists()
    before = sources()
    assert digest(subject) == sha256
    command = [sys.executable, "-B", str(RUN), "--new-binary", str(subject),
               "--expected-new-sha", sha256, "--output", str(output)]
    if args.preflight:
        return subprocess.run(command + ["--preflight"], cwd=ROOT).returncode

    record = dict(client_date="2026-11-18", head=subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        runner_sha256=digest(RUN), candidate_sha256=sha256, sources_before=before,
        status="running")
    def save():
        receipt.write_text(json.dumps(record, indent=2) + "\n", encoding="utf8")
    save()
    try:
        rc = subprocess.run(command, cwd=ROOT).returncode
        record.update(exit=rc, sources_after=sources(), raw_sha256=digest(output) if output.is_file() else None)
        unchanged = record["sources_after"] == before and digest(subject) == sha256
        record["status"] = "completed" if rc == 0 and unchanged else "invalidated" if not unchanged else "aborted"
        return 0 if record["status"] == "completed" else 1
    finally:
        save()

if __name__ == "__main__":
    sys.exit(main())
