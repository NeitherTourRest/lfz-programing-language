"""Run the frozen T13 protocol once against the isolated call-argument prototype."""
import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUN = ROOT / "benchmarks/t13/run.py"
SUBJECT = ROOT / "Temp/T26-perf/prototype-b.exe"
SHA256 = "7a14421320bc93d460d3be9abf48aa89b9c8091f7c6fc928ee11e860e52aa8ef"

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def sources():
    inputs = sorted((ROOT / "src").rglob("*.rs")) + [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in inputs}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--preflight", action="store_true")
    args = parser.parse_args()
    output = args.output.resolve()
    assert output.is_relative_to(ROOT / "Temp") and not output.exists()
    receipt = output.with_suffix(".receipt.json")
    assert not receipt.exists()
    before = sources()
    assert digest(SUBJECT) == SHA256
    command = [sys.executable, "-B", str(RUN), "--new-binary", str(SUBJECT),
               "--expected-new-sha", SHA256, "--output", str(output)]
    if args.preflight:
        return subprocess.run(command + ["--preflight"], cwd=ROOT).returncode

    record = dict(client_date="2026-11-17", head=subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        runner_sha256=digest(RUN), candidate_sha256=SHA256, sources_before=before,
        status="running")
    def save():
        receipt.write_text(json.dumps(record, indent=2) + "\n", encoding="utf8")
    save()
    try:
        rc = subprocess.run(command, cwd=ROOT).returncode
        record.update(exit=rc, sources_after=sources(), raw_sha256=digest(output) if output.is_file() else None)
        unchanged = record["sources_after"] == before and digest(SUBJECT) == SHA256
        record["status"] = "completed" if rc == 0 and unchanged else "invalidated" if not unchanged else "aborted"
        return 0 if record["status"] == "completed" else 1
    finally:
        save()

if __name__ == "__main__":
    sys.exit(main())
