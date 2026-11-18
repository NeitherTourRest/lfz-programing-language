"""Run and record the two T27 findIndex guide examples."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
GUIDE = "docs/guide/find-index.md"
MAIN = "docs/guide/examples/find-index.lfz"
BUDGET = "docs/guide/examples/find-index-budget.lfz"
CASES = [
    ("check", ["check", MAIN], 0),
    ("examples", ["run", MAIN], 0),
    ("budget", ["--json", "--max-steps", "200", "run", BUDGET], 2),
]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def guide_hashes():
    return {name: sha(ROOT / name) for name in (GUIDE, MAIN, BUDGET)}


def matches(name, stdout):
    if name == "check":
        return stdout == ""
    if name == "examples":
        return stdout.replace("\r\n", "\n") == "findIndex examples ok\n"
    try:
        err = json.loads(stdout)
    except ValueError:
        return False
    return (isinstance(err, dict) and err.get("ok") is False
            and err.get("error") == "ExecutionLimitError")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--sha256")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    docs = guide_hashes()
    if args.prepare_only:
        print(json.dumps({"prepared": len(CASES), "executed": False, "docs": docs}))
        return
    if not (args.binary and args.sha256 and args.output):
        parser.error("--binary, --sha256 and --output are required")
    binary = args.binary.resolve(strict=True)
    output = args.output.resolve()
    assert binary.is_relative_to(ROOT) and output.is_relative_to(ROOT)
    assert not output.exists() and sha(binary) == args.sha256.lower()
    rows = []
    for name, command, expected_exit in CASES:
        result = subprocess.run([binary, *command], cwd=ROOT,
                                capture_output=True, timeout=10)
        stdout = result.stdout.decode("utf-8")
        stderr = result.stderr.decode("utf-8")
        rows.append({"name": name, "args": command, "exit": result.returncode,
                     "stdout": stdout, "stderr": stderr,
                     "pass": result.returncode == expected_exit and not stderr
                             and matches(name, stdout)})
    passed = sum(row["pass"] for row in rows)
    evidence = {"binary_sha256": args.sha256.lower(), "binary_bytes": binary.stat().st_size,
                "final_binary_sha256": sha(binary), "docs": docs,
                "final_docs": guide_hashes(), "passed": passed, "runs": rows}
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x", encoding="utf-8") as log:
        json.dump(evidence, log, ensure_ascii=False, indent=2)
        log.write("\n")
    print(json.dumps({"passed": passed, "total": len(rows)}))
    assert evidence["binary_sha256"] == evidence["final_binary_sha256"]
    assert evidence["docs"] == evidence["final_docs"]
    raise SystemExit(passed != len(rows))


if __name__ == "__main__":
    main()
